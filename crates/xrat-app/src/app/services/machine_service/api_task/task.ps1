$ErrorActionPreference = 'Stop'
function Read-TaskXml($task) {
    $document = New-Object System.Xml.XmlDocument
    $document.XmlResolver = $null
    $document.LoadXml($task.Xml)
    return ,$document
}
function Resolve-TaskSid([string]$identity) {
    if ($identity -cmatch '^S-1-5-21-(?:[0-9]+-){3}[0-9]+$') { return $identity }
    $account = New-Object Security.Principal.NTAccount $identity
    $sid = $account.Translate([Security.Principal.SecurityIdentifier]).Value
    if ($sid -cnotmatch '^S-1-5-21-(?:[0-9]+-){3}[0-9]+$') {
        throw 'The API task principal is not an ordinary account SID; it was preserved.'
    }
    return $sid
}
function Assert-OwnedTask($task, $request) {
    $document = Read-TaskXml $task
    $principal = @($document.Task.Principals.Principal)
    $actions = @($document.Task.Actions.Exec)
    $triggers = @($document.Task.Triggers.LogonTrigger)
    $operator = if ($principal.Count -eq 1) { Resolve-TaskSid $principal[0].UserId } else { $null }
    $triggerOperator = if ($triggers.Count -eq 1) { Resolve-TaskSid $triggers[0].UserId } else { $null }
    if ($principal.Count -ne 1 -or $actions.Count -ne 1 -or $triggers.Count -ne 1 -or
        @($document.Task.Actions.ChildNodes).Count -ne 1 -or
        @($document.Task.Triggers.ChildNodes).Count -ne 1 -or
        $document.Task.RegistrationInfo.URI -cne $request.uri -or
        $principal[0].LogonType -cne 'InteractiveToken' -or
        $principal[0].RunLevel -cne 'LeastPrivilege' -or
        $triggerOperator -cne $operator -or
        -not [String]::Equals($actions[0].Command, $request.command, [StringComparison]::OrdinalIgnoreCase) -or
        -not [String]::Equals($actions[0].WorkingDirectory, $request.directory, [StringComparison]::OrdinalIgnoreCase) -or
        $actions[0].Arguments -cne $request.arguments) {
        $observed = @{
            principal_count = $principal.Count; action_count = $actions.Count; trigger_count = $triggers.Count
            uri_matches = $document.Task.RegistrationInfo.URI -ceq $request.uri
            logon = [string]$principal[0].LogonType; run_level = [string]$principal[0].RunLevel
            operator = $operator; trigger_operator = $triggerOperator
            command_matches = [String]::Equals($actions[0].Command, $request.command, [StringComparison]::OrdinalIgnoreCase)
            directory_matches = [String]::Equals($actions[0].WorkingDirectory, $request.directory, [StringComparison]::OrdinalIgnoreCase)
            arguments_match = $actions[0].Arguments -ceq $request.arguments
        } | ConvertTo-Json -Compress
        throw "The reserved API task has a different identity or action; it was preserved. Observed: $observed"
    }
    if ($null -ne $request.operator -and $operator -cne $request.operator) {
        throw 'The API task belongs to another operator; it was preserved.'
    }
    $expectedText = "O:BAG:BAD:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FRFX;;;$operator)"
    $expected = New-Object System.Security.AccessControl.RawSecurityDescriptor $expectedText
    $security = New-Object System.Security.AccessControl.RawSecurityDescriptor ($task.GetSecurityDescriptor(7))
    $protected = [System.Security.AccessControl.ControlFlags]::DiscretionaryAclProtected
    if ($security.Owner.Value -ne $expected.Owner.Value -or
        $security.Group.Value -ne $expected.Group.Value -or
        ($security.ControlFlags -band $protected) -eq 0 -or
        $security.DiscretionaryAcl.Count -ne $expected.DiscretionaryAcl.Count) {
        throw 'The reserved API task has an unexpected owner or ACL; it was preserved.'
    }
    $seen = @{}
    foreach ($entry in $security.DiscretionaryAcl) {
        if ($seen.ContainsKey($entry.SecurityIdentifier.Value)) {
            throw 'The reserved API task has duplicate permissions; it was preserved.'
        }
        $seen[$entry.SecurityIdentifier.Value] = $true
        $matches = @($expected.DiscretionaryAcl | Where-Object {
            $_.SecurityIdentifier.Value -eq $entry.SecurityIdentifier.Value -and
            $_.AccessMask -eq $entry.AccessMask -and $_.AceType -eq $entry.AceType -and
            $_.AceFlags -eq $entry.AceFlags
        })
        if ($matches.Count -ne 1) {
            throw 'The reserved API task has unexpected permissions; it was preserved.'
        }
    }
}
try {
    $scheduler = New-Object -ComObject Schedule.Service
    $scheduler.Connect()
    $root = $scheduler.GetFolder('\')
    $existing = $null
    try {
        $existing = $root.GetTask($request.name)
    } catch {
        $exception = $_.Exception
        while ($null -ne $exception.InnerException) { $exception = $exception.InnerException }
        if ($exception.HResult -ne -2147024894) { throw }
    }
    if ($null -ne $existing) { Assert-OwnedTask $existing $request }
    if ($request.operation -in @('pause', 'inspect')) {
        $state = 0
        if ($null -ne $existing) {
            if ($existing.Enabled) { $state = $state -bor 2 }
            if ($existing.GetInstances(0).Count -gt 0) { $state = $state -bor 1 }
            if ($request.operation -eq 'inspect') { exit (16 + $state) }
            $existing.Enabled = $false
            $existing.Stop(0)
            $deadline = [DateTime]::UtcNow.AddSeconds(20)
            while ($existing.GetInstances(0).Count -gt 0) {
                if ([DateTime]::UtcNow -gt $deadline) { throw 'API task did not stop; upgrade refused.' }
                Start-Sleep -Milliseconds 100
            }
        }
        exit (16 + $state)
    } elseif ($request.operation -in @('resume_run', 'resume_idle', 'resume_run_disabled', 'resume_disabled')) {
        if ($null -ne $existing) {
            $enabled = $request.operation -in @('resume_run', 'resume_idle')
            if ($request.operation -in @('resume_run', 'resume_run_disabled')) {
                $existing.Enabled = $true
                try { $null = $existing.Run($null) } finally { $existing.Enabled = $enabled }
            } else { $existing.Enabled = $enabled }
        }
    } elseif ($request.operation -eq 'remove') {
        if ($null -ne $existing) {
            $existing.Enabled = $false
            if ($existing.GetInstances(0).Count -gt 0) { $existing.Stop(0) }
            $root.DeleteTask($request.name, 0)
        }
    } elseif ($request.operation -eq 'install') {
        $registered = $root.RegisterTask($request.name, $request.xml, 54, $request.operator, $null, 3, $request.descriptor)
        Assert-OwnedTask $registered $request
    } else {
        throw 'Unsupported API task operation.'
    }
    exit 0
} catch {
    $message = $_.Exception.Message
    if ($message.Length -gt 2048) { $message = $message.Substring(0, 2048) }
    [Console]::Error.WriteLine($message)
    exit 1
}
