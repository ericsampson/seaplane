Short help:

```console
$ seaplane restrict set -h
Set a restriction

Usage: seaplane[EXE] restrict set [OPTIONS] <API> <DIRECTORY>

Arguments:
  <API>        The API of the restricted directory
  <DIRECTORY>  The restricted directory

Options:
  -B, --base64            The directory is already encoded in URL safe Base64
  -v, --verbose...        Display more verbose output
      --format <FORMAT>   Change the output format [default: table] [possible values: table, json]
  -q, --quiet...          Suppress output at a specific level and below
      --color <COLOR>     Should the output include color? [default: auto] [possible values: always, ansi, auto, never]
  -D, --decode            Decode the directories before printing them
      --no-color          Do not color output (alias for --color=never)
      --no-decode         Print directories without decoding them
  -A, --api-key <STRING>  The API key associated with a Seaplane account used to access Seaplane API endpoints [env: SEAPLANE_API_KEY]
  -S, --stateless         Ignore local state files, do not read from or write to them
  -h, --help              Print help (see more with '--help')
  -V, --version           Print version

RESTRICTION DETAILS:
      --provider <PROVIDER>
          A provider where the data placement is allowed (supports comma separated list, or multiple uses) [default: all] [aliases: providers] [possible values: aws, azure, digitalocean, equinix, gcp, all]
      --region <REGION>
          A region where the data placement is allowed (supports comma separated list, or multiple uses) (See REGION SPEC below) [default: all] [aliases: regions] [possible values: xa, xc, xe, xf, xn, xo, xq, xs, xu, all]
      --exclude-provider <PROVIDER>
          A provider where the data placement is *NOT* allowed (supports comma separated list, or multiple uses) [aliases: exclude-providers] [possible values: aws, azure, digitalocean, equinix, gcp, all]
      --exclude-region <REGION>
          A region where the data placement is *NOT* allowed (supports comma separated list, or multiple uses) (See REGION SPEC below) [aliases: exclude-regions] [possible values: xa, xc, xe, xf, xn, xo, xq, xs, xu, all]

REGION SPEC

    The regions are based on ISO 3166 alpha-2 continent codes with a few additions to capture
    regulatory differences along with some more intuitive or common aliases. The currently
    supported mappings are:

    XA => Asia
    XC => PRC => PeoplesRepublicofChina
    XE => EU  => Europe
    XF => Africa
    XN => NAmerica => NorthAmerica
    XO => Oceania
    XQ => Antarctica
    XS => SAmerica => SouthAmerica
    XU => UK => UnitedKingdom

    This list is subject to change or expand.

```

Long Help:

```console
$ seaplane restrict set --help
Set a restriction

Usage: seaplane[EXE] restrict set [OPTIONS] <API> <DIRECTORY>

Arguments:
  <API>
          The API of the restricted directory

  <DIRECTORY>
          The restricted directory

Options:
  -B, --base64
          The directory is already encoded in URL safe Base64

  -v, --verbose...
          Display more verbose output
          
          More uses displays more verbose output
              -v:  Display debug info
              -vv: Display trace info

      --format <FORMAT>
          Change the output format
          
          [default: table]
          [possible values: table, json]

  -q, --quiet...
          Suppress output at a specific level and below
          
          More uses suppresses higher levels of output
              -q:   Only display WARN messages and above
              -qq:  Only display ERROR messages
              -qqq: Suppress all output

      --color <COLOR>
          Should the output include color?
          
          [default: auto]
          [possible values: always, ansi, auto, never]

  -D, --decode
          Decode the directories before printing them
          
          Binary values will be written directly to standard output (which may do strange
          things to your terminal)

      --no-color
          Do not color output (alias for --color=never)

      --no-decode
          Print directories without decoding them

  -A, --api-key <STRING>
          The API key associated with a Seaplane account used to access Seaplane API endpoints
          
          The value provided here will override any provided in any configuration files.
          A CLI provided value also overrides any environment variables.
          One can use a special value of '-' to signal the value should be read from STDIN.
          
          [env: SEAPLANE_API_KEY]

  -S, --stateless
          Ignore local state files, do not read from or write to them

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

RESTRICTION DETAILS:
      --provider <PROVIDER>
          A provider where the data placement is allowed
          
          Multiple items can be passed as a comma separated list, or by using the argument
          multiple times.
          
          [default: all]
          [aliases: providers]
          [possible values: aws, azure, digitalocean, equinix, gcp, all]

      --region <REGION>
          A region where the data placement is allowed (See REGION SPEC below)
          
          Multiple items can be passed as a comma separated list, or by using the argument
          multiple times.
          
          [default: all]
          [aliases: regions]
          [possible values: xa, xc, xe, xf, xn, xo, xq, xs, xu, all]

      --exclude-provider <PROVIDER>
          A provider where the data placement is *NOT* allowed
          
          This will override any values given to --provider
          
          Multiple items can be passed as a comma separated list, or by using the argument
          multiple times.
          
          [aliases: exclude-providers]
          [possible values: aws, azure, digitalocean, equinix, gcp, all]

      --exclude-region <REGION>
          A region  where the data placement is *NOT* allowed (See REGION SPEC below)
          
          This will override any values given to --region
          
          Multiple items can be passed as a comma separated list, or by using the argument
          multiple times.
          
          [aliases: exclude-regions]
          [possible values: xa, xc, xe, xf, xn, xo, xq, xs, xu, all]

REGION SPEC

    The regions are based on ISO 3166 alpha-2 continent codes with a few additions to capture
    regulatory differences along with some more intuitive or common aliases. The currently
    supported mappings are:

    XA => Asia
    XC => PRC => PeoplesRepublicofChina
    XE => EU  => Europe
    XF => Africa
    XN => NAmerica => NorthAmerica
    XO => Oceania
    XQ => Antarctica
    XS => SAmerica => SouthAmerica
    XU => UK => UnitedKingdom

    This list is subject to change or expand.

```
