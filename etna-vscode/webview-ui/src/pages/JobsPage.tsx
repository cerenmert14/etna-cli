import { useState, useEffect, useRef } from 'react';
import { vscode, JobInfo, onMessage } from '../api/vscodeApi';

interface Props {
  jobs: JobInfo[];
  onRefresh: () => void;
}

function JobsPage({ jobs, onRefresh }: Props) {
  const [expandedJob, setExpandedJob] = useState<string | null>(null);
  const [jobLogs, setJobLogs] = useState<Record<string, string[]>>({});
  const [loadingLogs, setLoadingLogs] = useState<string | null>(null);
  const logsEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unsubscribe = onMessage((message) => {
      if (message.type === 'jobLogs') {
        const { id, logs } = message.data as { id: string; logs: string[] };
        setJobLogs(prev => ({ ...prev, [id]: logs }));
        setLoadingLogs(null);
      }
    });
    return unsubscribe;
  }, []);

  // Auto-scroll to bottom when logs update
  useEffect(() => {
    if (expandedJob && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [jobLogs, expandedJob]);

  const handleCancel = (id: string) => {
    if (confirm('Cancel this job?')) {
      vscode.postMessage({ type: 'cancelJob', id });
    }
  };

  const handleViewLogs = (id: string) => {
    if (expandedJob === id) {
      setExpandedJob(null);
    } else {
      setExpandedJob(id);
      setLoadingLogs(id);
      vscode.postMessage({ type: 'getJobLogs', id });
    }
  };

  const refreshLogs = (id: string) => {
    setLoadingLogs(id);
    vscode.postMessage({ type: 'getJobLogs', id });
  };

  const handleViewMetrics = (id: string) => {
    vscode.postMessage({ type: 'getJobMetrics', id });
  };

  const getStatusClass = (status: string) => {
    switch (status) {
      case 'running':
        return 'status-running';
      case 'completed':
        return 'status-completed';
      case 'failed':
        return 'status-failed';
      case 'pending':
        return 'status-pending';
      case 'cancelled':
        return 'status-cancelled';
      default:
        return '';
    }
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleString();
  };

  const formatDuration = (job: JobInfo) => {
    if (!job.started_at) return '-';
    const start = new Date(job.started_at).getTime();
    const end = job.completed_at
      ? new Date(job.completed_at).getTime()
      : Date.now();
    const duration = Math.floor((end - start) / 1000);

    if (duration < 60) return `${duration}s`;
    if (duration < 3600) return `${Math.floor(duration / 60)}m ${duration % 60}s`;
    return `${Math.floor(duration / 3600)}h ${Math.floor((duration % 3600) / 60)}m`;
  };

  // Sort jobs by created_at descending
  const sortedJobs = [...jobs].sort((a, b) =>
    new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );

  return (
    <div>
      <div className="toolbar">
        <button className="secondary" onClick={onRefresh}>Refresh</button>
        <span style={{ color: 'var(--vscode-descriptionForeground)', marginLeft: '10px' }}>
          Auto-refreshes every 5 seconds
        </span>
      </div>

      {sortedJobs.length === 0 ? (
        <div className="empty-state">
          <p>No jobs found.</p>
          <p>Run an experiment to create a job.</p>
        </div>
      ) : (
        <table>
          <thead>
            <tr>
              <th></th>
              <th>ID</th>
              <th>Type</th>
              <th>Status</th>
              <th>Duration</th>
              <th>Created</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {sortedJobs.map((job) => {
              const isExpanded = expandedJob === job.id;
              const logs = jobLogs[job.id] || [];

              return (
                <>
                  <tr key={job.id}>
                    <td>
                      <button
                        className="expand-btn"
                        onClick={() => handleViewLogs(job.id)}
                        title={isExpanded ? 'Hide logs' : 'Show logs'}
                      >
                        {isExpanded ? '▼' : '▶'}
                      </button>
                    </td>
                    <td>
                      <code title={job.id}>{job.id.substring(0, 8)}...</code>
                    </td>
                    <td>{job.job_type}</td>
                    <td>
                      <span className={`status-badge ${getStatusClass(job.status)}`}>
                        {job.status}
                      </span>
                      {job.error && (
                        <div style={{
                          color: 'var(--vscode-errorForeground)',
                          fontSize: '11px',
                          marginTop: '4px'
                        }}>
                          {job.error}
                        </div>
                      )}
                    </td>
                    <td>{formatDuration(job)}</td>
                    <td>{formatDate(job.created_at)}</td>
                    <td>
                      <div className="actions">
                        <button className="secondary" onClick={() => handleViewLogs(job.id)}>
                          {isExpanded ? 'Hide Logs' : 'Logs'}
                        </button>
                        {(job.status === 'completed' || job.status === 'failed') && (
                          <button className="secondary" onClick={() => handleViewMetrics(job.id)}>
                            Metrics
                          </button>
                        )}
                        {(job.status === 'running' || job.status === 'pending') && (
                          <button className="danger" onClick={() => handleCancel(job.id)}>
                            Cancel
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                  {isExpanded && (
                    <tr key={`${job.id}-logs`} className="logs-row">
                      <td colSpan={7}>
                        <div className="logs-panel">
                          <div className="logs-header">
                            <strong>Job Logs</strong>
                            <button
                              className="small secondary"
                              onClick={() => refreshLogs(job.id)}
                              disabled={loadingLogs === job.id}
                            >
                              {loadingLogs === job.id ? 'Loading...' : 'Refresh'}
                            </button>
                          </div>
                          {logs.length === 0 ? (
                            <div className="no-logs">
                              {loadingLogs === job.id ? 'Loading logs...' : 'No logs available'}
                            </div>
                          ) : (
                            <pre className="logs-content">
                              {logs.join('\n')}
                              <div ref={logsEndRef} />
                            </pre>
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

export default JobsPage;
