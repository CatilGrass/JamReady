use crate::cli_commands::client::{exec, print_client_result, StructArgs};

pub async fn client_struct(args: StructArgs) {

    let mut env_flags = String::new();
    let mut flags = String::new();

    if args.local { env_flags.push_str("l"); }
    if args.remote { env_flags.push_str("r"); }
    if env_flags.is_empty() {
        env_flags = "lr".to_string();
    }

    if args.remote_zero { flags.push_str("z"); }
    if args.remote_held { flags.push_str("h"); }
    if args.remote_updated { flags.push_str("u"); }
    if args.local_untracked { flags.push_str("n"); }
    if args.local_completed { flags.push_str("c"); }
    if args.local_removed { flags.push_str("d"); }
    if args.remote_locked { flags.push_str("g"); }
    if args.moved { flags.push_str("m"); }
    if args.remote_other { flags.push_str("e"); }
    if flags.is_empty() {
        flags = "zhundecmg".to_string();
    }

    print_client_result(exec(vec!["struct".to_string(), env_flags, flags]).await);
}