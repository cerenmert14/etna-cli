import { useState, useEffect } from 'react';
import { vscode, ExperimentInfo, TestInfo, TestDefinition, onMessage } from '../api/vscodeApi';
import TestEditor from '../components/TestEditor';

interface Props {
  experiments: ExperimentInfo[];
  loading: boolean;
  onRefresh: () => void;
  experimentTests: Record<string, TestInfo[]>;
  onFetchTests: (experimentName: string) => void;
}

interface EditingTest {
  experimentName: string;
  testName: string;
  tests: TestDefinition[];
  isNew: boolean;
}

function ExperimentsPage({ experiments, loading, onRefresh, experimentTests, onFetchTests }: Props) {
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newExperimentName, setNewExperimentName] = useState('');
  const [expandedExperiment, setExpandedExperiment] = useState<string | null>(null);
  const [selectedTests, setSelectedTests] = useState<Record<string, Set<string>>>({});
  const [editingTest, setEditingTest] = useState<EditingTest | null>(null);
  const [loadingTest, setLoadingTest] = useState<string | null>(null);

  useEffect(() => {
    const unsubscribe = onMessage((message) => {
      if (message.type === 'testContent') {
        const { experimentName, testName, tests } = message.data as {
          experimentName: string;
          testName: string;
          tests: TestDefinition[];
        };
        setEditingTest({ experimentName, testName, tests, isNew: false });
        setLoadingTest(null);
      }
    });
    return unsubscribe;
  }, []);

  const handleCreate = () => {
    if (!newExperimentName.trim()) return;
    vscode.postMessage({
      type: 'createExperiment',
      name: newExperimentName.trim(),
    });
    setNewExperimentName('');
    setShowCreateForm(false);
  };

  const handleDelete = (name: string) => {
    if (confirm(`Delete experiment "${name}"?`)) {
      vscode.postMessage({ type: 'deleteExperiment', name });
    }
  };

  const handleRun = (name: string, runAll: boolean = false) => {
    const tests = experimentTests[name] || [];
    const selected = selectedTests[name] || new Set<string>();

    let testsToRun: string[];
    if (runAll || selected.size === 0) {
      testsToRun = tests.map(t => t.name);
    } else {
      testsToRun = Array.from(selected);
    }

    vscode.postMessage({
      type: 'runExperiment',
      name,
      tests: testsToRun,
    });
  };

  const toggleExpanded = (name: string) => {
    if (expandedExperiment === name) {
      setExpandedExperiment(null);
    } else {
      setExpandedExperiment(name);
      if (!experimentTests[name]) {
        onFetchTests(name);
      }
    }
  };

  const toggleTestSelection = (experimentName: string, testName: string) => {
    setSelectedTests(prev => {
      const current = prev[experimentName] || new Set<string>();
      const updated = new Set(current);
      if (updated.has(testName)) {
        updated.delete(testName);
      } else {
        updated.add(testName);
      }
      return { ...prev, [experimentName]: updated };
    });
  };

  const selectAllTests = (experimentName: string) => {
    const tests = experimentTests[experimentName] || [];
    setSelectedTests(prev => ({
      ...prev,
      [experimentName]: new Set(tests.map(t => t.name)),
    }));
  };

  const deselectAllTests = (experimentName: string) => {
    setSelectedTests(prev => ({
      ...prev,
      [experimentName]: new Set<string>(),
    }));
  };

  const handleEditTest = (experimentName: string, testName: string) => {
    setLoadingTest(`${experimentName}/${testName}`);
    vscode.postMessage({ type: 'getTest', experimentName, testName });
  };

  const handleNewTest = (experimentName: string) => {
    const testName = prompt('Enter test name (without .json extension):');
    if (!testName?.trim()) return;
    setEditingTest({
      experimentName,
      testName: testName.trim(),
      tests: [{
        language: '',
        workload: '',
        trials: 10,
        timeout: 60,
        mutations: ['base'],
        cross: false,
        params: {},
        tasks: [{ strategy: '', property: '' }],
      }],
      isNew: true,
    });
  };

  const handleSaveTest = (tests: TestDefinition[]) => {
    if (!editingTest) return;
    vscode.postMessage({
      type: 'saveTest',
      experimentName: editingTest.experimentName,
      testName: editingTest.testName,
      tests,
    });
    setEditingTest(null);
  };

  const handleDeleteTest = () => {
    if (!editingTest || editingTest.isNew) return;
    if (!confirm(`Delete test "${editingTest.testName}"?`)) return;
    vscode.postMessage({
      type: 'deleteTest',
      experimentName: editingTest.experimentName,
      testName: editingTest.testName,
    });
    setEditingTest(null);
  };

  if (loading) {
    return <div className="loading">Loading experiments...</div>;
  }

  // Show test editor if editing
  if (editingTest) {
    return (
      <TestEditor
        experimentName={editingTest.experimentName}
        testName={editingTest.testName}
        initialTests={editingTest.tests}
        isNew={editingTest.isNew}
        onSave={handleSaveTest}
        onCancel={() => setEditingTest(null)}
        onDelete={editingTest.isNew ? undefined : handleDeleteTest}
      />
    );
  }

  return (
    <div>
      <div className="toolbar">
        <button onClick={() => setShowCreateForm(true)}>+ New Experiment</button>
        <div className="toolbar-spacer" />
        <button className="secondary" onClick={onRefresh}>Refresh</button>
      </div>

      {showCreateForm && (
        <div className="card">
          <div className="card-title">Create New Experiment</div>
          <div className="form-group">
            <label htmlFor="expName">Experiment Name</label>
            <input
              id="expName"
              type="text"
              value={newExperimentName}
              onChange={(e) => setNewExperimentName(e.target.value)}
              placeholder="my-experiment"
              onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
            />
          </div>
          <div className="actions">
            <button onClick={handleCreate}>Create</button>
            <button className="secondary" onClick={() => setShowCreateForm(false)}>Cancel</button>
          </div>
        </div>
      )}

      {experiments.length === 0 ? (
        <div className="empty-state">
          <p>No experiments found.</p>
          <button onClick={() => setShowCreateForm(true)}>Create your first experiment</button>
        </div>
      ) : (
        <table>
          <thead>
            <tr>
              <th></th>
              <th>Name</th>
              <th>Workloads</th>
              <th>Path</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {experiments.map((exp) => {
              const tests = experimentTests[exp.name] || [];
              const selected = selectedTests[exp.name] || new Set<string>();
              const isExpanded = expandedExperiment === exp.name;

              return (
                <>
                  <tr key={exp.name}>
                    <td>
                      <button
                        className="expand-btn"
                        onClick={() => toggleExpanded(exp.name)}
                        title={isExpanded ? 'Collapse tests' : 'Show tests'}
                      >
                        {isExpanded ? '▼' : '▶'}
                      </button>
                    </td>
                    <td>
                      <strong>{exp.name}</strong>
                    </td>
                    <td>{exp.workloads?.length || 0} workload(s)</td>
                    <td>
                      <code title={exp.path}>{truncatePath(exp.path)}</code>
                    </td>
                    <td>
                      <div className="actions">
                        <button
                          onClick={() => handleRun(exp.name, false)}
                          title={selected.size > 0 ? `Run ${selected.size} selected test(s)` : 'Run all tests'}
                        >
                          {selected.size > 0 ? `Run (${selected.size})` : 'Run All'}
                        </button>
                        <button className="danger" onClick={() => handleDelete(exp.name)}>Delete</button>
                      </div>
                    </td>
                  </tr>
                  {isExpanded && (
                    <tr key={`${exp.name}-tests`} className="tests-row">
                      <td colSpan={5}>
                        <div className="tests-panel">
                          <div className="tests-header">
                            <strong>Tests</strong>
                            <span className="tests-actions">
                              <button className="small" onClick={() => handleNewTest(exp.name)}>+ New Test</button>
                              {tests.length > 0 && (
                                <>
                                  <button className="small" onClick={() => selectAllTests(exp.name)}>Select All</button>
                                  <button className="small" onClick={() => deselectAllTests(exp.name)}>Deselect All</button>
                                </>
                              )}
                            </span>
                          </div>
                          {tests.length === 0 ? (
                            <div className="no-tests">No tests found. Create your first test!</div>
                          ) : (
                            <div className="tests-list">
                              {tests.map((test) => (
                                <label key={test.name} className="test-item">
                                  <input
                                    type="checkbox"
                                    checked={selected.has(test.name)}
                                    onChange={() => toggleTestSelection(exp.name, test.name)}
                                  />
                                  <span>{test.name}</span>
                                  <button
                                    className="small secondary"
                                    onClick={(e) => {
                                      e.preventDefault();
                                      handleEditTest(exp.name, test.name);
                                    }}
                                    disabled={loadingTest === `${exp.name}/${test.name}`}
                                  >
                                    {loadingTest === `${exp.name}/${test.name}` ? '...' : 'Edit'}
                                  </button>
                                </label>
                              ))}
                            </div>
                          )}
                        </div>
                      </td>
                    </tr>
                  )}
                </>
              );
            })}
          </tbody>
        </table>
      )}
    </div>
  );
}

function truncatePath(path: string, maxLength = 40): string {
  if (path.length <= maxLength) return path;
  const parts = path.split('/');
  if (parts.length <= 2) return path;
  return `.../${parts.slice(-2).join('/')}`;
}

export default ExperimentsPage;
