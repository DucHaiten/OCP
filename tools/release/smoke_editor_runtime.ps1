param(
    [Parameter(Mandatory = $true)]
    [string]$LspPath,
    [Parameter(Mandatory = $true)]
    [string]$DapPath,
    [Parameter(Mandatory = $true)]
    [string]$LspFixture,
    [Parameter(Mandatory = $true)]
    [string]$DapFixture,
    [Parameter(Mandatory = $true)]
    [string]$WorkspaceFolder
)

$ErrorActionPreference = "Stop"

function Start-JsonRpcProcess {
    param([string]$ExePath)

    $startInfo = New-Object System.Diagnostics.ProcessStartInfo
    $startInfo.FileName = $ExePath
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardInput = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    $startInfo.CreateNoWindow = $true

    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $startInfo
    [void]$process.Start()

    $encoding = New-Object System.Text.UTF8Encoding($false)
    $reader = New-Object System.IO.StreamReader($process.StandardOutput.BaseStream, $encoding, $false)
    $writer = New-Object System.IO.StreamWriter($process.StandardInput.BaseStream, $encoding)
    $writer.AutoFlush = $true

    return @{
        Process = $process
        Reader = $reader
        Writer = $writer
    }
}

function Send-ProtocolMessage {
    param($Writer, [hashtable]$Message)

    $json = $Message | ConvertTo-Json -Depth 100 -Compress
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($json)
    $Writer.Write("Content-Length: {0}`r`n`r`n" -f $bytes.Length)
    $Writer.Write($json)
    $Writer.Flush()
}

function Read-ProtocolMessage {
    param($Reader)

    $contentLength = 0
    while ($true) {
        $line = $Reader.ReadLine()
        if ($null -eq $line) {
            throw "protocol stream closed unexpectedly"
        }
        if ($line -eq "") {
            break
        }
        if ($line.StartsWith("Content-Length:")) {
            $contentLength = [int]($line.Substring("Content-Length:".Length).Trim())
        }
    }

    if ($contentLength -le 0) {
        throw "invalid Content-Length from protocol stream"
    }

    $buffer = New-Object char[] $contentLength
    $offset = 0
    while ($offset -lt $contentLength) {
        $read = $Reader.Read($buffer, $offset, $contentLength - $offset)
        if ($read -le 0) {
            throw "protocol payload truncated"
        }
        $offset += $read
    }

    $raw = -join $buffer
    return $raw | ConvertFrom-Json
}

function Wait-ProtocolMessage {
    param($Reader, [scriptblock]$Predicate)

    while ($true) {
        $message = Read-ProtocolMessage -Reader $Reader
        if (& $Predicate $message) {
            return $message
        }
    }
}

function Assert-True {
    param([bool]$Condition, [string]$Message)
    if (-not $Condition) {
        throw $Message
    }
}

function To-FileUri {
    param([string]$PathValue)
    return ([System.Uri] (Resolve-Path $PathValue).Path).AbsoluteUri
}

$lsp = Start-JsonRpcProcess -ExePath $LspPath
try {
    Send-ProtocolMessage -Writer $lsp.Writer -Message @{
        jsonrpc = "2.0"
        id = 1
        method = "initialize"
        params = @{
            processId = $PID
            rootUri = (To-FileUri -PathValue $WorkspaceFolder)
            capabilities = @{}
        }
    }
    $init = Wait-ProtocolMessage -Reader $lsp.Reader -Predicate {
        param($Message)
        return $Message.id -eq 1
    }
    Assert-True ($init.result.capabilities.hoverProvider -eq $true) "LSP initialize missing hoverProvider"

    $lspUri = To-FileUri -PathValue $LspFixture
    Send-ProtocolMessage -Writer $lsp.Writer -Message @{
        jsonrpc = "2.0"
        id = 2
        method = "textDocument/hover"
        params = @{
            textDocument = @{ uri = $lspUri }
            position = @{ line = 5; character = 14 }
        }
    }
    $hover = Wait-ProtocolMessage -Reader $lsp.Reader -Predicate {
        param($Message)
        return $Message.id -eq 2
    }
    Assert-True ($hover.result.contents.value -match "helper") "LSP hover response missing helper detail"

    Send-ProtocolMessage -Writer $lsp.Writer -Message @{
        jsonrpc = "2.0"
        id = 3
        method = "textDocument/semanticTokens/full"
        params = @{
            textDocument = @{ uri = $lspUri }
        }
    }
    $semantic = Wait-ProtocolMessage -Reader $lsp.Reader -Predicate {
        param($Message)
        return $Message.id -eq 3
    }
    Assert-True ($semantic.result.data.Count -gt 0) "LSP semantic tokens stream was empty"

    Send-ProtocolMessage -Writer $lsp.Writer -Message @{
        jsonrpc = "2.0"
        id = 4
        method = "shutdown"
        params = @{}
    }
    [void](Wait-ProtocolMessage -Reader $lsp.Reader -Predicate {
        param($Message)
        return $Message.id -eq 4
    })
    Send-ProtocolMessage -Writer $lsp.Writer -Message @{
        jsonrpc = "2.0"
        method = "exit"
        params = @{}
    }
}
finally {
    if (-not $lsp.Process.HasExited) {
        $lsp.Process.Kill()
    }
}

$dap = Start-JsonRpcProcess -ExePath $DapPath
try {
    Send-ProtocolMessage -Writer $dap.Writer -Message @{
        seq = 1
        type = "request"
        command = "initialize"
        arguments = @{
            clientID = "ocl-smoke"
            adapterID = "ocp-ocl"
        }
    }
    $dapInit = Wait-ProtocolMessage -Reader $dap.Reader -Predicate {
        param($Message)
        return $Message.type -eq "response" -and $Message.request_seq -eq 1
    }
    Assert-True ($dapInit.success -eq $true) "DAP initialize failed"

    Send-ProtocolMessage -Writer $dap.Writer -Message @{
        seq = 2
        type = "request"
        command = "launch"
        arguments = @{
            program = (Resolve-Path $DapFixture).Path
            workspaceFolder = (Resolve-Path $WorkspaceFolder).Path
            workspaceHash = "smoke"
            runId = "smoke-run"
        }
    }
    $dapLaunch = Wait-ProtocolMessage -Reader $dap.Reader -Predicate {
        param($Message)
        return $Message.type -eq "response" -and $Message.request_seq -eq 2
    }
    Assert-True ($dapLaunch.success -eq $true) "DAP launch failed"
    [void](Wait-ProtocolMessage -Reader $dap.Reader -Predicate {
        param($Message)
        return $Message.type -eq "event" -and $Message.event -eq "initialized"
    })

    Send-ProtocolMessage -Writer $dap.Writer -Message @{
        seq = 3
        type = "request"
        command = "setBreakpoints"
        arguments = @{
            source = @{ path = (Resolve-Path $DapFixture).Path }
            lines = @(6)
        }
    }
    $setBreakpoints = Wait-ProtocolMessage -Reader $dap.Reader -Predicate {
        param($Message)
        return $Message.type -eq "response" -and $Message.request_seq -eq 3
    }
    Assert-True ($setBreakpoints.success -eq $true) "DAP setBreakpoints failed"

    Send-ProtocolMessage -Writer $dap.Writer -Message @{
        seq = 4
        type = "request"
        command = "configurationDone"
        arguments = @{}
    }
    $configDone = Wait-ProtocolMessage -Reader $dap.Reader -Predicate {
        param($Message)
        return $Message.type -eq "response" -and $Message.request_seq -eq 4
    }
    Assert-True ($configDone.success -eq $true) "DAP configurationDone failed"
    [void](Wait-ProtocolMessage -Reader $dap.Reader -Predicate {
        param($Message)
        return $Message.type -eq "event" -and $Message.event -eq "stopped"
    })

    Send-ProtocolMessage -Writer $dap.Writer -Message @{
        seq = 5
        type = "request"
        command = "stackTrace"
        arguments = @{
            threadId = 1
        }
    }
    $stackTrace = Wait-ProtocolMessage -Reader $dap.Reader -Predicate {
        param($Message)
        return $Message.type -eq "response" -and $Message.request_seq -eq 5
    }
    Assert-True ($stackTrace.body.stackFrames.Count -gt 0) "DAP stackTrace returned no frames"

    Send-ProtocolMessage -Writer $dap.Writer -Message @{
        seq = 6
        type = "request"
        command = "disconnect"
        arguments = @{}
    }
    $disconnect = Wait-ProtocolMessage -Reader $dap.Reader -Predicate {
        param($Message)
        return $Message.type -eq "response" -and $Message.request_seq -eq 6
    }
    Assert-True ($disconnect.success -eq $true) "DAP disconnect failed"
}
finally {
    if (-not $dap.Process.HasExited) {
        $dap.Process.Kill()
    }
}

Write-Host "editor runtime smoke PASS"
