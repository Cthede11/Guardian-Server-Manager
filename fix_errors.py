#!/usr/bin/env python3
import re
import os

def fix_app_error_usage(file_path):
    """Fix AppError usage in a file"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Fix FileSystem errors
    content = re.sub(
        r'AppError::FileSystem\("([^"]+)"\)',
        r'AppError::FileSystemError {\n                message: "\1".to_string(),\n                path: "unknown".to_string(),\n                operation: "unknown".to_string(),\n            }',
        content
    )
    
    # Fix Process errors
    content = re.sub(
        r'AppError::Process\("([^"]+)"\)',
        r'AppError::ProcessError {\n                message: "\1".to_string(),\n                process_id: None,\n                operation: "unknown".to_string(),\n            }',
        content
    )
    
    # Fix Validation errors
    content = re.sub(
        r'AppError::Validation\("([^"]+)"\)',
        r'AppError::ValidationError {\n                message: "\1".to_string(),\n                field: "unknown".to_string(),\n                value: "unknown".to_string(),\n            }',
        content
    )
    
    # Fix ServerNotFound errors
    content = re.sub(
        r'AppError::ServerNotFound\(([^)]+)\)',
        r'AppError::ServerError {\n                message: "Server not found".to_string(),\n                server_id: \1,\n                operation: "get".to_string(),\n            }',
        content
    )
    
    # Fix Internal errors
    content = re.sub(
        r'AppError::Internal\("([^"]+)"\)',
        r'AppError::InternalError {\n                message: "\1".to_string(),\n                operation: "unknown".to_string(),\n            }',
        content
    )
    
    # Fix Authentication errors
    content = re.sub(
        r'AppError::Authentication\("([^"]+)"\)',
        r'AppError::AuthenticationError {\n                message: "\1".to_string(),\n                reason: crate::core::error_handler::AuthErrorReason::InternalError,\n            }',
        content
    )
    
    # Fix Authorization errors
    content = re.sub(
        r'AppError::Authorization\("([^"]+)"\)',
        r'AppError::AuthorizationError {\n                message: "\1".to_string(),\n                required_permission: "unknown".to_string(),\n                user_role: "unknown".to_string(),\n            }',
        content
    )
    
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)

def main():
    # Files to fix
    files_to_fix = [
        'hostd/src/core/file_manager.rs',
        'hostd/src/core/security.rs',
        'hostd/src/core/monitoring.rs',
        'hostd/src/core/websocket.rs',
    ]
    
    for file_path in files_to_fix:
        if os.path.exists(file_path):
            print(f"Fixing {file_path}...")
            fix_app_error_usage(file_path)
        else:
            print(f"File not found: {file_path}")

if __name__ == "__main__":
    main()
