use super::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(super) struct TaskSnapshot {
    principal: String,
    logon: String,
    run_level: String,
    command: String,
    arguments: String,
    owner: String,
    group: String,
    protected: bool,
    permissions: Vec<Permission>,
}
#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Permission {
    sid: String,
    mask: i64,
    flags: i64,
    kind: i64,
}

pub(super) fn snapshot() -> Option<TaskSnapshot> {
    let shell = xrat_support::windows::system_directory()
        .unwrap()
        .join("WindowsPowerShell/v1.0/powershell.exe");
    let output = Command::new(shell).args(["-NoProfile", "-NonInteractive", "-Command", r#"
$ErrorActionPreference = 'Stop'
try {
    $scheduler = New-Object -ComObject Schedule.Service
    $scheduler.Connect()
    try { $task = $scheduler.GetFolder('\').GetTask('XRAT-Machine-API') }
    catch {
        $errorValue = $_.Exception
        while ($null -ne $errorValue.InnerException) { $errorValue = $errorValue.InnerException }
        if ($errorValue.HResult -eq -2147024894) { Write-Output 'null'; exit 0 }
        throw
    }
    $xml = New-Object System.Xml.XmlDocument
    $xml.XmlResolver = $null
    $xml.LoadXml($task.Xml)
    $principal = [string]$xml.Task.Principals.Principal.UserId
    if ($principal -notmatch '^S-1-') {
        $principal = (New-Object Security.Principal.NTAccount $principal).Translate([Security.Principal.SecurityIdentifier]).Value
    }
    $sd = New-Object System.Security.AccessControl.RawSecurityDescriptor ($task.GetSecurityDescriptor(7))
    $permissions = @($sd.DiscretionaryAcl | ForEach-Object {
        @{ sid = $_.SecurityIdentifier.Value; mask = [long]$_.AccessMask; flags = [long]$_.AceFlags; kind = [long]$_.AceType }
    } | Sort-Object { $_.sid })
    @{
        principal = $principal
        logon = $xml.Task.Principals.Principal.LogonType
        run_level = if ($task.Definition.Principal.RunLevel -eq 0) { 'LeastPrivilege' } else { 'HighestAvailable' }
        command = $xml.Task.Actions.Exec.Command
        arguments = $xml.Task.Actions.Exec.Arguments
        owner = $sd.Owner.Value; group = $sd.Group.Value
        protected = (($sd.ControlFlags -band [System.Security.AccessControl.ControlFlags]::DiscretionaryAclProtected) -ne 0)
        permissions = $permissions
    } | ConvertTo-Json -Depth 5 -Compress
} catch { Write-Error $_; exit 1 }
"#]).output().unwrap();
    assert!(
        output.status.success(),
        "task inspection failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

pub(super) fn assert_operator_task(operator: &str, executable: &Path) -> TaskSnapshot {
    let snapshot = snapshot().expect("API task must be registered");
    assert_eq!(snapshot.principal, operator);
    assert_eq!(snapshot.logon, "InteractiveToken");
    assert_eq!(snapshot.run_level, "LeastPrivilege");
    assert_eq!(Path::new(&snapshot.command), executable);
    assert_eq!(
        snapshot.arguments,
        "--profile machine serve --host 127.0.0.1"
    );
    assert_eq!(snapshot.owner, "S-1-5-32-544");
    assert_eq!(snapshot.group, "S-1-5-32-544");
    assert!(snapshot.protected);
    assert_eq!(snapshot.permissions.len(), 3);
    assert_eq!(
        snapshot
            .permissions
            .iter()
            .map(|permission| &permission.sid)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        3
    );
    for permission in &snapshot.permissions {
        assert_eq!((permission.flags, permission.kind), (0, 0));
        assert_eq!(
            permission.mask,
            if permission.sid == operator {
                0x0012_00a9
            } else {
                assert!(matches!(
                    permission.sid.as_str(),
                    "S-1-5-18" | "S-1-5-32-544"
                ));
                0x001f_01ff
            }
        );
    }
    snapshot
}
