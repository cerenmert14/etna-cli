import { useState, useMemo } from 'react';
import { vscode, QueryResult } from '../api/vscodeApi';

interface Props {
  queryResult: QueryResult | null;
}

type SortDirection = 'asc' | 'desc';

function MetricsPage({ queryResult }: Props) {
  const [filter, setFilter] = useState('.[]');
  const [loading, setLoading] = useState(false);
  const [sortColumn, setSortColumn] = useState<string | null>(null);
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');
  const [columnFilters, setColumnFilters] = useState<Record<string, string>>({});
  const [viewMode, setViewMode] = useState<'table' | 'json'>('table');

  const handleQuery = () => {
    setLoading(true);
    vscode.postMessage({ type: 'queryMetrics', filter });
    setTimeout(() => setLoading(false), 5000);
  };

  const exampleQueries = [
    { label: 'All metrics', query: '.[]' },
    { label: 'Rust workloads', query: '.[] | select(.language == "Rust")' },
    { label: 'Failed tests', query: '.[] | select(.success == false)' },
    { label: 'Group by workload', query: 'group_by(.workload) | map({workload: .[0].workload, count: length})' },
    { label: 'Average time by language', query: 'group_by(.language) | map({language: .[0].language, avg_time: (map(.time) | add / length)})' },
  ];

  // Detect columns from data
  const columns = useMemo(() => {
    if (!queryResult?.metrics?.length) return [];
    const cols = new Set<string>();
    queryResult.metrics.forEach(m => {
      if (typeof m === 'object' && m !== null) {
        Object.keys(m).forEach(k => cols.add(k));
      }
    });
    // Prioritize common columns
    const priority = ['language', 'workload', 'status', 'time', 'trial', 'mutations', 'strategy', 'property'];
    const prioritized = priority.filter(c => cols.has(c));
    const rest = Array.from(cols).filter(c => !priority.includes(c)).sort();
    return [...prioritized, ...rest];
  }, [queryResult]);

  // Filter and sort metrics
  const processedMetrics = useMemo(() => {
    if (!queryResult?.metrics?.length) return [];

    let result = [...queryResult.metrics];

    // Apply column filters
    Object.entries(columnFilters).forEach(([col, filterValue]) => {
      if (filterValue.trim()) {
        const lowerFilter = filterValue.toLowerCase();
        result = result.filter(m => {
          const value = (m as Record<string, unknown>)[col];
          if (value === null || value === undefined) return false;
          return String(value).toLowerCase().includes(lowerFilter);
        });
      }
    });

    // Apply sorting
    if (sortColumn) {
      result.sort((a, b) => {
        const aVal = (a as Record<string, unknown>)[sortColumn];
        const bVal = (b as Record<string, unknown>)[sortColumn];

        if (aVal === null || aVal === undefined) return 1;
        if (bVal === null || bVal === undefined) return -1;

        let comparison = 0;
        if (typeof aVal === 'number' && typeof bVal === 'number') {
          comparison = aVal - bVal;
        } else {
          comparison = String(aVal).localeCompare(String(bVal));
        }

        return sortDirection === 'asc' ? comparison : -comparison;
      });
    }

    return result;
  }, [queryResult, columnFilters, sortColumn, sortDirection]);

  const handleSort = (column: string) => {
    if (sortColumn === column) {
      setSortDirection(prev => prev === 'asc' ? 'desc' : 'asc');
    } else {
      setSortColumn(column);
      setSortDirection('asc');
    }
  };

  const handleFilterChange = (column: string, value: string) => {
    setColumnFilters(prev => ({ ...prev, [column]: value }));
  };

  const formatCellValue = (column: string, value: unknown): string => {
    if (value === null || value === undefined) return '-';

    // Format time values (assumes nanoseconds in format "123ns")
    if (column === 'time' && typeof value === 'string') {
      const match = value.match(/^(\d+)ns$/);
      if (match) {
        const ns = parseInt(match[1], 10);
        if (ns >= 1e9) return `${(ns / 1e9).toFixed(2)}s`;
        if (ns >= 1e6) return `${(ns / 1e6).toFixed(2)}ms`;
        if (ns >= 1e3) return `${(ns / 1e3).toFixed(2)}µs`;
        return `${ns}ns`;
      }
      return value;
    }

    // Format arrays
    if (Array.isArray(value)) {
      return value.join(', ');
    }

    // Format objects
    if (typeof value === 'object') {
      return JSON.stringify(value);
    }

    return String(value);
  };

  const getStatusClass = (status: string) => {
    switch (status?.toLowerCase()) {
      case 'foundbug': return 'status-failed';
      case 'timedout': return 'status-pending';
      case 'aborted': return 'status-cancelled';
      case 'passed': return 'status-completed';
      default: return '';
    }
  };

  return (
    <div>
      <div className="card">
        <div className="card-title">Query Metrics</div>
        <p style={{ marginBottom: '10px', color: 'var(--vscode-descriptionForeground)' }}>
          Enter a JQ filter expression to query metrics from the store.
        </p>

        <div className="form-group">
          <label htmlFor="jqFilter">JQ Filter</label>
          <textarea
            id="jqFilter"
            rows={3}
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder='.[] | select(.language == "Rust")'
          />
        </div>

        <div className="toolbar">
          <button onClick={handleQuery} disabled={loading}>
            {loading ? 'Executing...' : 'Execute Query'}
          </button>
        </div>

        <div style={{ marginTop: '15px' }}>
          <span style={{ fontSize: '12px', color: 'var(--vscode-descriptionForeground)' }}>
            Example queries:
          </span>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '5px', marginTop: '5px' }}>
            {exampleQueries.map((example, i) => (
              <button
                key={i}
                className="secondary"
                style={{ fontSize: '11px', padding: '4px 8px' }}
                onClick={() => setFilter(example.query)}
              >
                {example.label}
              </button>
            ))}
          </div>
        </div>
      </div>

      {queryResult && (
        <div className="card">
          <div className="card-title" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span>Results ({processedMetrics.length} of {queryResult.metrics?.length || 0} items)</span>
            <div className="actions">
              <button
                className={`small ${viewMode === 'table' ? '' : 'secondary'}`}
                onClick={() => setViewMode('table')}
              >
                Table
              </button>
              <button
                className={`small ${viewMode === 'json' ? '' : 'secondary'}`}
                onClick={() => setViewMode('json')}
              >
                JSON
              </button>
            </div>
          </div>

          {queryResult.metrics && queryResult.metrics.length > 0 ? (
            viewMode === 'json' ? (
              <pre style={{ maxHeight: '400px', overflow: 'auto' }}>
                {JSON.stringify(processedMetrics, null, 2)}
              </pre>
            ) : (
              <div className="metrics-table-container">
                <table className="metrics-table">
                  <thead>
                    <tr>
                      {columns.map(col => (
                        <th
                          key={col}
                          onClick={() => handleSort(col)}
                          className="sortable-header"
                        >
                          <span>{col}</span>
                          {sortColumn === col && (
                            <span className="sort-indicator">
                              {sortDirection === 'asc' ? ' ▲' : ' ▼'}
                            </span>
                          )}
                        </th>
                      ))}
                    </tr>
                    <tr className="filter-row">
                      {columns.map(col => (
                        <th key={`filter-${col}`}>
                          <input
                            type="text"
                            placeholder="Filter..."
                            value={columnFilters[col] || ''}
                            onChange={(e) => handleFilterChange(col, e.target.value)}
                            className="column-filter"
                          />
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {processedMetrics.map((metric, idx) => (
                      <tr key={idx}>
                        {columns.map(col => {
                          const value = (metric as Record<string, unknown>)[col];
                          const displayValue = formatCellValue(col, value);
                          const isStatus = col === 'status';

                          return (
                            <td key={col} title={displayValue}>
                              {isStatus ? (
                                <span className={`status-badge ${getStatusClass(String(value))}`}>
                                  {displayValue}
                                </span>
                              ) : (
                                <span className="cell-value">{displayValue}</span>
                              )}
                            </td>
                          );
                        })}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )
          ) : (
            <div className="empty-state">
              <p>No results found for this query.</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default MetricsPage;
