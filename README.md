# onepass
Light, ergonomic and portable terminal password manager.
Get to your passwords super quick and hassle free!

## Lightweight & Secure.
Operates on a single encrypted file.

## Portable.
Want to have your passwords on a portable disk? Simply copy the `onepass` file to it and specify the path!
e.g `onepass ls -l /run/media/<user>/<device>`

## Commands.

```shell
COMMANDS:
    new    [OPTIONS] - create a new resource
    get    [OPTIONS] - get a resource by its name
    rm     [OPTIONS] - remove a resource
    ls     [OPTIONS] - list resources
    update [OPTIONS] - update a resouruce - its name, username or password
    suggest - suggest a new strong password

OPTIONS:
    -l, --location - specify the location of the source file
```

## Development
Enter the development environment with `nix develop`.
