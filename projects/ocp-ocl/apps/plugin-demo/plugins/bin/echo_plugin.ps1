$line = [Console]::In.ReadLine()
if ([string]::IsNullOrWhiteSpace($line)) {
  Write-Output '{"id":"1","kind":"deferred","reason":"RC-PLUGIN-PROTOCOL-ERROR","payload":""}'
  exit 0
}

Write-Output '{"id":"1","kind":"ok","reason":"RC-ADAPTER-FAILED","payload":"echo-plugin"}'
