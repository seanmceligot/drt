extern crate assert_cmd;
extern crate predicates;

#[test]
fn test_t() {
    {
   let mut cmd = assert_cmd::Command::cargo_bin("met").unwrap();
   let _assert = cmd.arg("t").arg("Cargo.toml").arg("/root/outfile")
        .assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("Insufficient Privileges"));
    }
    {
   let mut cmd = assert_cmd::Command::cargo_bin("met").unwrap();
   let _assert = cmd.arg("t").arg("Cargo.toml").arg("/root/nodir/outfile")
        .assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("Insufficient Privileges"));
        }
        {
   let mut cmd = assert_cmd::Command::cargo_bin("met").unwrap();
   let args = & ["--", "x", "ls", "-l", "Cargo.toml"];
   let _assert = cmd.args(args)
        .assert()
        .success()
        .code(0)
        .stdout(predicates::str::contains("WOULD"));
        }
    ()
}

