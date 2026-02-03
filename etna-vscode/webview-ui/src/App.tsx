import { useState, useEffect, useCallback } from 'react';
import { vscode, onMessage, ExperimentInfo, JobInfo, QueryResult, WebviewMessage, TestInfo } from './api/vscodeApi';
import ExperimentsPage from './pages/ExperimentsPage';
import JobsPage from './pages/JobsPage';
import MetricsPage from './pages/MetricsPage';

type Tab = 'experiments' | 'jobs' | 'metrics';

function App() {
  const [currentTab, setCurrentTab] = useState<Tab>('experiments');
  const [experiments, setExperiments] = useState<ExperimentInfo[]>([]);
  const [jobs, setJobs] = useState<JobInfo[]>([]);
  const [queryResult, setQueryResult] = useState<QueryResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [experimentTests, setExperimentTests] = useState<Record<string, TestInfo[]>>({});

  // Handle messages from extension
  const handleMessage = useCallback((message: WebviewMessage) => {
    switch (message.type) {
      case 'experiments':
        setExperiments(message.data as ExperimentInfo[] || []);
        setLoading(false);
        break;
      case 'jobs':
        setJobs(message.data as JobInfo[] || []);
        break;
      case 'queryResult':
        setQueryResult(message.data as QueryResult);
        break;
      case 'jobMetrics': {
        const metricsData = message.data as { id: string; metrics: QueryResult };
        setQueryResult(metricsData.metrics);
        setCurrentTab('metrics');
        break;
      }
      case 'tests': {
        const testsData = message.data as { experimentName: string; tests: TestInfo[] };
        setExperimentTests(prev => ({
          ...prev,
          [testsData.experimentName]: testsData.tests,
        }));
        break;
      }
      case 'error':
        setError(message.message || 'Unknown error');
        setTimeout(() => setError(null), 5000);
        break;
      case 'healthCheck':
        // Server is healthy
        break;
    }
  }, []);

  useEffect(() => {
    const unsubscribe = onMessage(handleMessage);

    // Request initial data
    vscode.postMessage({ type: 'getExperiments' });
    vscode.postMessage({ type: 'getJobs' });
    vscode.postMessage({ type: 'healthCheck' });

    return unsubscribe;
  }, [handleMessage]);

  // Auto-refresh jobs when on jobs tab
  useEffect(() => {
    if (currentTab !== 'jobs') return;

    const interval = setInterval(() => {
      vscode.postMessage({ type: 'getJobs' });
    }, 5000);

    return () => clearInterval(interval);
  }, [currentTab]);

  const refreshExperiments = () => {
    vscode.postMessage({ type: 'getExperiments' });
  };

  const refreshJobs = () => {
    vscode.postMessage({ type: 'getJobs' });
  };

  const fetchTests = (experimentName: string) => {
    vscode.postMessage({ type: 'getTests', experimentName });
  };

  return (
    <div className="app">
      <h1>Etna Dashboard</h1>

      {error && <div className="error">{error}</div>}

      <div className="tabs">
        <button
          className={`tab ${currentTab === 'experiments' ? 'active' : ''}`}
          onClick={() => setCurrentTab('experiments')}
        >
          Experiments
        </button>
        <button
          className={`tab ${currentTab === 'jobs' ? 'active' : ''}`}
          onClick={() => setCurrentTab('jobs')}
        >
          Jobs
        </button>
        <button
          className={`tab ${currentTab === 'metrics' ? 'active' : ''}`}
          onClick={() => setCurrentTab('metrics')}
        >
          Metrics
        </button>
      </div>

      <div className="content">
        {currentTab === 'experiments' && (
          <ExperimentsPage
            experiments={experiments}
            loading={loading}
            onRefresh={refreshExperiments}
            experimentTests={experimentTests}
            onFetchTests={fetchTests}
          />
        )}
        {currentTab === 'jobs' && (
          <JobsPage
            jobs={jobs}
            onRefresh={refreshJobs}
          />
        )}
        {currentTab === 'metrics' && (
          <MetricsPage
            queryResult={queryResult}
          />
        )}
      </div>
    </div>
  );
}

export default App;
