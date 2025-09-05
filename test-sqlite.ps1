# Test SQLite functionality
Write-Host "Testing SQLite connection..."

# Try to create a simple SQLite database
$dbPath = "test.db"
try {
    # Remove existing test database
    if (Test-Path $dbPath) {
        Remove-Item $dbPath -Force
    }
    
    # Test if we can create a file in the current directory
    "test" | Out-File -FilePath "test.txt" -Encoding UTF8
    if (Test-Path "test.txt") {
        Write-Host "✓ Can create files in current directory"
        Remove-Item "test.txt" -Force
    } else {
        Write-Host "✗ Cannot create files in current directory"
    }
    
    # Check if SQLite is available
    $sqliteTest = @"
import sqlite3
import os
try:
    conn = sqlite3.connect('test.db')
    conn.execute('CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)')
    conn.execute('INSERT INTO test (name) VALUES (?)', ('test',))
    conn.commit()
    conn.close()
    print("✓ SQLite works with Python")
    os.remove('test.db')
except Exception as e:
    print(f"✗ SQLite error: {e}")
"@
    
    $sqliteTest | python
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Python SQLite test passed"
    } else {
        Write-Host "✗ Python SQLite test failed"
    }
    
} catch {
    Write-Host "Error: $($_.Exception.Message)"
}

Write-Host "SQLite test complete!"
