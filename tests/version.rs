mod common;
use crate::common::*;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use renku_cli::cli::BuildInfo;

#[test]
#[ignore = "needs a server"]
fn version_json_cmd() -> Result<()> {
    let mut cmd = mk_cmd()?;
    let assert = cmd.args(["-f", "json"]).arg("version").assert();

    let res = serde_json::from_slice::<serde_json::Value>(
        assert.success().stderr("").get_output().stdout.as_slice(),
    )?;
    assert!(res.get("client").is_some());
    assert!(res.get("server").is_some());
    assert!(res["server"].get("data").is_some());
    assert!(res["server"].get("search").is_some());
    Ok(())
}

#[test]
#[ignore = "needs a server"]
fn version_default_cmd() -> Result<()> {
    let cmd = mk_cmd()?.arg("version").unwrap();
    let info = BuildInfo::default();
    cmd.assert()
        .stderr("")
        .stdout(predicate::str::is_match(format!("Version: {}", info.build_version)).unwrap());
    Ok(())
}
