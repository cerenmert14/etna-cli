import { useState, useEffect, useRef, useCallback, SetStateAction } from 'react';
import { TestDefinition } from '../api/vscodeApi';

const MAX_HISTORY = 50;

interface Props {
  experimentName: string;
  testName: string;
  initialTests: TestDefinition[];
  onSave: (tests: TestDefinition[]) => void;
  onCancel: () => void;
  onDelete?: () => void;
  isNew?: boolean;
}

function TestEditor({ experimentName, testName, initialTests, onSave, onCancel, onDelete, isNew }: Props) {
  const [tests, setTests] = useState<TestDefinition[]>(initialTests);
  const [activeTestIndex, setActiveTestIndex] = useState(0);
  const [paramsText, setParamsText] = useState(
    initialTests[0]?.params ? JSON.stringify(initialTests[0].params, null, 2) : '{}'
  );
  const historyRef = useRef<TestDefinition[][]>([]);
  const futureRef = useRef<TestDefinition[][]>([]);

  const setTestsWithHistory = useCallback((update: SetStateAction<TestDefinition[]>) => {
    setTests(prev => {
      historyRef.current = [...historyRef.current.slice(-(MAX_HISTORY - 1)), prev];
      futureRef.current = [];
      return typeof update === 'function' ? update(prev) : update;
    });
  }, []);

  const undo = useCallback(() => {
    if (historyRef.current.length === 0) return;
    setTests(prev => {
      const previous = historyRef.current[historyRef.current.length - 1];
      historyRef.current = historyRef.current.slice(0, -1);
      futureRef.current = [...futureRef.current, prev];
      setActiveTestIndex(idx => Math.min(idx, previous.length - 1));
      return previous;
    });
  }, []);

  const redo = useCallback(() => {
    if (futureRef.current.length === 0) return;
    setTests(prev => {
      const next = futureRef.current[futureRef.current.length - 1];
      futureRef.current = futureRef.current.slice(0, -1);
      historyRef.current = [...historyRef.current, prev];
      setActiveTestIndex(idx => Math.min(idx, next.length - 1));
      return next;
    });
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'z') {
        e.preventDefault();
        e.stopPropagation();
        if (e.shiftKey) {
          redo();
        } else {
          undo();
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown, true);
    return () => window.removeEventListener('keydown', handleKeyDown, true);
  }, [undo, redo]);

  useEffect(() => {
    setTests(initialTests);
    setActiveTestIndex(0);
    setParamsText(initialTests[0]?.params ? JSON.stringify(initialTests[0].params, null, 2) : '{}');
    historyRef.current = [];
    futureRef.current = [];
  }, [initialTests]);

  useEffect(() => {
    const current = tests[activeTestIndex];
    setParamsText(current?.params ? JSON.stringify(current.params, null, 2) : '{}');
  }, [activeTestIndex, tests]);

  const activeTest = tests[activeTestIndex] || {
    language: '',
    workload: '',
    trials: 10,
    timeout: 60,
    mutations: [],
    cross: false,
    params: {},
    tasks: [],
  };

  const updateActiveTest = (updates: Partial<TestDefinition>) => {
    setTestsWithHistory(prev => {
      const updated = [...prev];
      updated[activeTestIndex] = { ...activeTest, ...updates };
      return updated;
    });
  };

  const addTest = () => {
    setTestsWithHistory(prev => [...prev, {
      language: '',
      workload: '',
      trials: 10,
      timeout: 60,
      mutations: [],
      cross: false,
      params: {},
      tasks: [],
    }]);
    setActiveTestIndex(tests.length);
  };

  const removeTest = (index: number) => {
    if (tests.length <= 1) return;
    setTestsWithHistory(prev => prev.filter((_, i) => i !== index));
    if (activeTestIndex >= tests.length - 1) {
      setActiveTestIndex(Math.max(0, tests.length - 2));
    }
  };

  const handleMutationsChange = (value: string) => {
    const mutations = value.split(',').map(m => m.trim()).filter(m => m);
    updateActiveTest({ mutations });
  };

  const handleParamsChange = (value: string) => {
    setParamsText(value);
    try {
      const params = value ? JSON.parse(value) : {};
      updateActiveTest({ params });
    } catch {
      // Invalid JSON — let user keep typing, don't update model
    }
  };

  const addTask = () => {
    updateActiveTest({
      tasks: [...(activeTest.tasks || []), {}]
    });
  };

  const updateTask = (taskIndex: number, field: string, value: string) => {
    const newTasks = [...(activeTest.tasks || [])];
    newTasks[taskIndex] = { ...newTasks[taskIndex], [field]: value };
    updateActiveTest({ tasks: newTasks });
  };

  const removeTask = (taskIndex: number) => {
    updateActiveTest({
      tasks: (activeTest.tasks || []).filter((_, i) => i !== taskIndex)
    });
  };

  const addTaskField = (taskIndex: number) => {
    const newTasks = [...(activeTest.tasks || [])];
    let fieldName = 'field';
    let counter = 1;
    while (newTasks[taskIndex][fieldName] !== undefined) {
      fieldName = `field${counter++}`;
    }
    newTasks[taskIndex] = { ...newTasks[taskIndex], [fieldName]: '' };
    updateActiveTest({ tasks: newTasks });
  };

  const removeTaskField = (taskIndex: number, key: string) => {
    const newTasks = [...(activeTest.tasks || [])];
    const { [key]: _, ...rest } = newTasks[taskIndex];
    newTasks[taskIndex] = rest;
    updateActiveTest({ tasks: newTasks });
  };

  const renameTaskField = (taskIndex: number, oldKey: string, newKey: string) => {
    if (!newKey || newKey === oldKey) return;
    const newTasks = [...(activeTest.tasks || [])];
    const task = newTasks[taskIndex];
    if (newKey in task && newKey !== oldKey) return; // duplicate key guard
    const entries = Object.entries(task).map(([k, v]) =>
      k === oldKey ? [newKey, v] : [k, v]
    );
    newTasks[taskIndex] = Object.fromEntries(entries);
    updateActiveTest({ tasks: newTasks });
  };

  const handleSave = () => {
    // Validate
    const validTests = tests.filter(t => t.language && t.workload);
    if (validTests.length === 0) {
      alert('Please fill in at least language and workload for each test');
      return;
    }
    onSave(validTests);
  };

  return (
    <div className="test-editor">
      <div className="test-editor-header">
        <h3>{isNew ? 'Create New Test' : `Edit Test: ${testName}`}</h3>
        <span className="experiment-name">Experiment: {experimentName}</span>
      </div>

      {tests.length > 1 && (
        <div className="test-tabs">
          {tests.map((_, i) => (
            <button
              key={i}
              className={`test-tab ${i === activeTestIndex ? 'active' : ''}`}
              onClick={() => setActiveTestIndex(i)}
            >
              Test {i + 1}
              {tests.length > 1 && (
                <span
                  className="remove-test"
                  onClick={(e) => { e.stopPropagation(); removeTest(i); }}
                >
                  &times;
                </span>
              )}
            </button>
          ))}
          <button className="test-tab add-tab" onClick={addTest}>+</button>
        </div>
      )}

      <div className="test-editor-form">
        <div className="form-row">
          <div className="form-group">
            <label>Language *</label>
            <input
              type="text"
              value={activeTest.language}
              onChange={(e) => updateActiveTest({ language: e.target.value })}
              placeholder="e.g., Rust, OCaml, Racket"
            />
          </div>
          <div className="form-group">
            <label>Workload *</label>
            <input
              type="text"
              value={activeTest.workload}
              onChange={(e) => updateActiveTest({ workload: e.target.value })}
              placeholder="e.g., BST, RBT, STLC"
            />
          </div>
        </div>

        <div className="form-row">
          <div className="form-group">
            <label>Trials</label>
            <input
              type="number"
              value={activeTest.trials}
              onChange={(e) => updateActiveTest({ trials: parseInt(e.target.value) || 1 })}
              min={1}
            />
          </div>
          <div className="form-group">
            <label>Timeout (seconds)</label>
            <input
              type="number"
              value={activeTest.timeout}
              onChange={(e) => updateActiveTest({ timeout: parseFloat(e.target.value) || 60 })}
              min={1}
              step={0.1}
            />
          </div>
          <div className="form-group checkbox-group">
            <label>
              <input
                type="checkbox"
                checked={activeTest.cross || false}
                onChange={(e) => updateActiveTest({ cross: e.target.checked })}
              />
              Cross-language
            </label>
          </div>
        </div>

        <div className="form-group">
          <label>Mutations (comma-separated)</label>
          <input
            type="text"
            value={(activeTest.mutations || []).join(', ')}
            onChange={(e) => handleMutationsChange(e.target.value)}
            placeholder="e.g., base, mutation1, mutation2"
          />
        </div>

        <div className="form-group">
          <label>Params (JSON)</label>
          <textarea
            rows={3}
            value={paramsText}
            onChange={(e) => handleParamsChange(e.target.value)}
            placeholder='{"key": "value"}'
          />
        </div>

        <div className="form-group">
          <label>
            Tasks
            <button className="small secondary" onClick={addTask} style={{ marginLeft: '10px' }}>
              + Add Task
            </button>
          </label>
          {(activeTest.tasks || []).length === 0 ? (
            <p className="help-text">No tasks defined. Add a task to specify key-value pairs.</p>
          ) : (
            <div className="tasks-list">
              {(activeTest.tasks || []).map((task, taskIndex) => {
                const entries = Object.entries(task);
                return (
                  <div key={taskIndex} className="task-card">
                    <div className="task-card-header">
                      <span className="task-label">Task {taskIndex + 1}</span>
                      <div className="task-card-actions">
                        <button className="small secondary" onClick={() => addTaskField(taskIndex)}>+ Field</button>
                        <button className="small danger" onClick={() => removeTask(taskIndex)}>&times;</button>
                      </div>
                    </div>
                    <div className="task-fields">
                      {entries.length === 0 ? (
                        <p className="help-text">No fields. Click '+ Field' to add key-value pairs.</p>
                      ) : (
                        entries.map(([key, value], fieldIndex) => (
                          <div key={`${taskIndex}-${fieldIndex}`} className="task-field-row">
                            <input
                              className="task-key-input"
                              type="text"
                              value={key}
                              onChange={(e) => renameTaskField(taskIndex, key, e.target.value)}
                              placeholder="key"
                            />
                            <span className="task-field-separator">=</span>
                            <input
                              className="task-value-input"
                              type="text"
                              value={value || ''}
                              onChange={(e) => updateTask(taskIndex, key, e.target.value)}
                              placeholder="value"
                            />
                            <button className="small danger" onClick={() => removeTaskField(taskIndex, key)}>&times;</button>
                          </div>
                        ))
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>

      <div className="test-editor-actions">
        <button onClick={handleSave}>Save</button>
        <button className="secondary" onClick={onCancel}>Cancel</button>
        {onDelete && !isNew && (
          <button className="danger" onClick={onDelete}>Delete Test</button>
        )}
        {tests.length === 1 && (
          <button className="secondary" onClick={addTest}>+ Add Variant</button>
        )}
      </div>
    </div>
  );
}

export default TestEditor;
