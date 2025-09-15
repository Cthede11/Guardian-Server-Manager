import React, { useState } from 'react';
import { useModpackStore } from '../../store/modpack';
import type { ModpackCompatibility, CompatibilityIssue } from '../../lib/types/modpack';

export const CompatibilityChecker: React.FC = () => {
  const { checkCompatibility, loading } = useModpackStore();
  const [compatibility, setCompatibility] = useState<ModpackCompatibility | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleCheckCompatibility = async () => {
    // This would be implemented with actual modpack data
    setError('Compatibility checking not yet implemented');
  };

  const getSeverityColor = (severity: CompatibilityIssue['severity']) => {
    switch (severity) {
      case 'critical': return 'text-red-600 bg-red-50 border-red-200';
      case 'warning': return 'text-yellow-600 bg-yellow-50 border-yellow-200';
      case 'info': return 'text-blue-600 bg-blue-50 border-blue-200';
      default: return 'text-gray-600 bg-gray-50 border-gray-200';
    }
  };

  const getSeverityIcon = (severity: CompatibilityIssue['severity']) => {
    switch (severity) {
      case 'critical': return '🚫';
      case 'warning': return '⚠️';
      case 'info': return 'ℹ️';
      default: return '❓';
    }
  };

  return (
    <div className="max-w-4xl mx-auto">
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-2xl font-bold text-gray-900 mb-6">Compatibility Checker</h2>
        
        <div className="mb-6">
          <p className="text-gray-600 mb-4">
            Check the compatibility of your modpack to ensure all mods work together properly.
          </p>
          <button
            onClick={handleCheckCompatibility}
            disabled={loading}
            className="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {loading ? 'Checking...' : 'Check Compatibility'}
          </button>
        </div>

        {error && (
          <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
            <div className="flex">
              <div className="flex-shrink-0">
                <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                  <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                </svg>
              </div>
              <div className="ml-3">
                <h3 className="text-sm font-medium text-red-800">Error</h3>
                <div className="mt-2 text-sm text-red-700">
                  <p>{error}</p>
                </div>
              </div>
            </div>
          </div>
        )}

        {compatibility && (
          <div className="space-y-6">
            {/* Overall Status */}
            <div className={`p-4 rounded-lg border ${
              compatibility.report.is_compatible 
                ? 'bg-green-50 border-green-200' 
                : 'bg-red-50 border-red-200'
            }`}>
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  {compatibility.report.is_compatible ? (
                    <svg className="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                    </svg>
                  ) : (
                    <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                    </svg>
                  )}
                </div>
                <div className="ml-3">
                  <h3 className={`text-sm font-medium ${
                    compatibility.report.is_compatible ? 'text-green-800' : 'text-red-800'
                  }`}>
                    {compatibility.report.is_compatible ? 'Compatible' : 'Incompatible'}
                  </h3>
                  <div className={`mt-2 text-sm ${
                    compatibility.report.is_compatible ? 'text-green-700' : 'text-red-700'
                  }`}>
                    <p>
                      {compatibility.report.is_compatible 
                        ? 'Your modpack is compatible and ready to use!'
                        : `Found ${compatibility.report.issues.length} compatibility issues that need to be resolved.`
                      }
                    </p>
                  </div>
                </div>
              </div>
            </div>

            {/* Issues */}
            {compatibility.report.issues.length > 0 && (
              <div>
                <h4 className="text-lg font-medium text-gray-900 mb-4">Issues</h4>
                <div className="space-y-3">
                  {compatibility.report.issues.map((issue: any, index: number) => (
                    <div
                      key={index}
                      className={`p-4 rounded-lg border ${getSeverityColor(issue.severity)}`}
                    >
                      <div className="flex items-start">
                        <div className="flex-shrink-0">
                          <span className="text-lg">{getSeverityIcon(issue.severity)}</span>
                        </div>
                        <div className="ml-3">
                          <h5 className="text-sm font-medium">
                            {issue.type.replace('_', ' ').toUpperCase()}
                          </h5>
                          <p className="mt-1 text-sm">
                            {issue.reason || `Issue with ${issue.mod_id}`}
                          </p>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Warnings */}
            {compatibility.report.warnings.length > 0 && (
              <div>
                <h4 className="text-lg font-medium text-gray-900 mb-4">Warnings</h4>
                <div className="space-y-3">
                  {compatibility.report.warnings.map((warning: any, index: number) => (
                    <div
                      key={index}
                      className={`p-4 rounded-lg border ${getSeverityColor(warning.severity)}`}
                    >
                      <div className="flex items-start">
                        <div className="flex-shrink-0">
                          <span className="text-lg">{getSeverityIcon(warning.severity)}</span>
                        </div>
                        <div className="ml-3">
                          <h5 className="text-sm font-medium">
                            {warning.type.replace('_', ' ').toUpperCase()}
                          </h5>
                          <p className="mt-1 text-sm">
                            {warning.reason || `Warning for ${warning.mod_id}`}
                          </p>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Recommendations */}
            {compatibility.report.recommendations.length > 0 && (
              <div>
                <h4 className="text-lg font-medium text-gray-900 mb-4">Recommendations</h4>
                <div className="space-y-2">
                  {compatibility.report.recommendations.map((recommendation: any, index: number) => (
                    <div key={index} className="flex items-start">
                      <div className="flex-shrink-0">
                        <svg className="h-5 w-5 text-blue-400" viewBox="0 0 20 20" fill="currentColor">
                          <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
                        </svg>
                      </div>
                      <div className="ml-3">
                        <p className="text-sm text-gray-700">{String(recommendation)}</p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
