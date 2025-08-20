// Auto-generated
use jam_ready::utils::text_process::parse_colored_text;

/// From ./_text/add.txt
pub const ADD: &'static str = "
ARGUMENTS: [green]<PATH>[/]     Virtual File path.

  OPTIONS: [gray]<--get/-g>[/] Lock after adding file.";

/// From ./_text/client/help.txt
pub const CLIENT_HELP: &'static str = "
ONLINE COMMANDS:

    [yellow]add[/]       [green]<PATH>[/]
        Alias: new, create
        Add a empty Virtual File
        [green]Learn more:[/] [yellow]jam[/] doc [cyan]add[/]

    [yellow]remove[/]    [green]<FROM_SEARCH>[/]
        Alias: rm, delete, del
        Remove a Virtual File(s)
        [green]Learn more:[/] [yellow]jam[/] doc [cyan]remove[/]

    [yellow]move[/]      [green]<FROM_SEARCH> <TO_SEARCH>[/]
        Alias: mv, rename
        Move Virtual File(s)
        [green]Learn more:[/] [yellow]jam[/] doc [cyan]move[/]

    [yellow]get/throw[/] [green]<FROM_SEARCH>[/]
        Alias: g/t, lock/unlock or release
        Lock/Unlock Virtual File(s)
        [green]Learn more:[/] [yellow]jam[/] doc [cyan]ownership[/]

    [yellow]view[/]      [green]<FROM_SEARCH>[/]
        Alias: v, download, dl
        Download Virtual File(s)
        [green]Learn more:[/] [yellow]jam[/] doc [cyan]view[/]

    [yellow]rollback[/]  [green]<FROM_SEARCH> <TO_VERSION>[/]
        Alias: rb, restore
        Change version of Virtual File(s).
        [green]Learn more:[/] [yellow]jam[/] doc [cyan]rollback[/]

    [yellow]commit[/]    [green]<INFO?>[/]
        Alias: cmt, save, sv
        Upload all held and modified Local File(s).

    [yellow]struct[/]    [green]<FILITERS?>[/]
        Alias: tree, list, ls
        Display the workspace file struct.
        [green]Learn more:[/] [yellow]jam[/] doc [cyan]struct[/]

    [yellow]redirect[/]
        Alias: red
        Redirect to a new network address.

    [yellow]update[/]
        Alias: sync
        Sync the local workspace struct with the remote.

OFFLINE COMMANDS:

    [yellow]query[/] [green]<QUERY_TYPE?> <QUERY_ITEMS?>[/]
        Alias: q
        Query some data.

    [yellow]param[/] [green]<KEY> <VALUE?>[/]
        Alias: set.
        Query or set a param.
";

/// From ./_text/move.txt
pub const MOVE: &'static str = "
ARGUMENTS: [green]<FROM_SEARCH>[/] Files to be moved
           [green]<TO_SEARCH>[/]   Dest path.

  OPTIONS: [gray]<--get/-g>[/]    Attempt to lock before moving.
           [gray]<--local/l>[/]   Move local files.

[green]Learn more:[/] [yellow]jam[/] doc [cyan]search_rule[/]";

/// From ./_text/ownership.txt
pub const OWNERSHIP: &'static str = "
 COMMANDS: [yellow]get[/]           Attempt to acquire file lock
           [yellow]throw[/]         Attempt to release file lock

ARGUMENTS: [green]<FROM_SEARCH>[/] Files to be locked

  OPTIONS: [gray]<--longer/-l>[/] Is it a long-term lock? [red](Only get command)[/]

[green]Learn more:[/] [yellow]jam[/] doc [cyan]search_rule[/]";

/// From ./_text/remove.txt
pub const REMOVE: &'static str = "
ARGUMENTS: [green]<FROM_SEARCH>[/] Files to be removed

  OPTIONS: [gray]<--get/-g>[/]    Attempt to lock before removal

[green]Learn more:[/] [yellow]jam[/] doc [cyan]search_rule[/]";

/// From ./_text/rollback.txt
pub const ROLLBACK: &'static str = "
ARGUMENTS: [green]<FROM_SEARCH>[/] Files to be rolled back
           [green]<TO_VERSION>[/]  Target version to roll back to

  OPTIONS: [gray]<--get/-g>[/]    Attempt to lock before rollback
  OPTIONS: [gray]<--back/-b>[/]    Download the rolled-back files

[green]Learn more:[/] [yellow]jam[/] doc [cyan]search_rule[/]";

/// From ./_text/search_rule.txt
pub const SEARCH_RULE: &'static str = "
FROM_SEARCH:
    Direct path specification
    [cyan]\"Documents/FileName.txt\"[/]

    Reference by filename
    [cyan]\"[/][green]:[/][cyan]FileName.txt\"[/]

    Reference parameter content
    [cyan]\"MyFile[/][green]?[/][cyan]\"[/]

    Regex pattern matching
    [cyan]\"Documents/[/][gray].[/][green]*[/][cyan]\"[/]

TO_SEARCH:
    Direct path specification
    [cyan]\"Documents/FileName_Renamed.txt\"[/]

    Reference relative to FROM
    [cyan]\"[/][gray].[/][cyan]/FileName_Renamed.txt\"[/] -> [cyan]\"Documents/FileName_Renamed.txt\"[/]

    Bind new directory
    [cyan]\"Documents/Backup/\"[/] -> [cyan]\"Documents/Backup/FileName_Renamed.txt\"[/]";

/// From ./_text/server/help.txt
pub const SERVER_HELP: &'static str = "
JAM SERVER COMMANDS:

    [yellow]run[/] Run VCS Server.

    [yellow]add/remove/list/query/set[/] View or modify workspace configs.
";

/// From ./_text/setup/help.txt
pub const SETUP_HELP: &'static str = "
[green] Setup a SERVER WORKSPACE: [/]
    ~# [yellow]jam[/] setup [green]<WORKSPACE_NAME>[/]

[green] Join a CLIENT WORKSPACE: [/]

    [gray]// Use Workspace Name[/]
    ~# [yellow]jam[/] login [green]<LOGIN_CODE>[/] --workspace [green]<WORKSPACE_NAME>[/]

    [gray]// Use Ip Address[/]
    ~# [yellow]jam[/] login [green]<LOGIN_CODE>[/] --target [green]<TARGET_ADDR>[/]
";

/// From ./_text/struct.txt
pub const STRUCT: &'static str = "
OPTIONS: [gray]<--local>[/]                 Local files
         [gray]<--remote>[/]                Remote files
         [gray]<--zero/-z/--empty/--new>[/] Zero-version, empty, newly created files
         [gray]<--updated/-u>[/]            Updated files
         [gray]<--held/-h>[/]               Files held by self
         [gray]<--lock/g>[/]                Locked files
         [gray]<--removed/d>[/]             Removed files
         [gray]<--untracked/n>[/]           Untracked files
         [gray]<--moved/m>[/]               Moved files
         [gray]<--other/e>[/]               Other files";

/// From ./_text/view.txt
pub const VIEW: &'static str = "
ARGUMENTS: [green]<FROM_SEARCH>[/]  Downloaded and viewed files

  OPTIONS: [gray]<--get/-g>[/]     Attempt to lock after download.
           [gray]<--version/-v>[/] Specify the file version to download.

[green]Learn more:[/] [yellow]jam[/] doc [cyan]search_rule[/]";

pub fn get_help_docs(name: &str) -> String {
    parse_colored_text(match name {
        "client_help" => CLIENT_HELP,
        "remove" => REMOVE,
        "search_rule" => SEARCH_RULE,
        "server_help" => SERVER_HELP,
        "view" => VIEW,
        "ownership" => OWNERSHIP,
        "struct" => STRUCT,
        "move" => MOVE,
        "setup_help" => SETUP_HELP,
        "add" => ADD,
        "rollback" => ROLLBACK,
        _ => "",
    }.trim())
}
