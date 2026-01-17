pub const MSG_NO_RESOURCES: &str = "No resources saved - create one with `onepass new`";
pub const MSG_COMMAND_GET: &str = "Get resource: e.g - onepass get <resource>";
pub const MSG_COMMAND_RM: &str = "Remove resource: e.g - onepass rm <resource>";
pub const MSG_COMMAND_UPDATE: &str = "Update resource: e.g - onepass update <resource>";
pub const MSG_HELP: &str = "COMMANDS:
    new    [OPTIONS] - create a new resource
    get    [OPTIONS] - get a resource by its name
    rm     [OPTIONS] - remove a resource
    ls     [OPTIONS] - list resources
    update [OPTIONS] - update a resouruce - its name, username or password
    suggest - suggest a new strong password

    OPTIONS:
    -l, --location - specify the location of the source file
";

pub const RESERVED_NONCE: &str = "nonce";
pub const RESERVED_RESOURCE: &str = "resource";
