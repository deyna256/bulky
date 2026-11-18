# Security policy

Report vulnerabilities privately to
[literallybugcreator@gmail.com](mailto:literallybugcreator@gmail.com)
or use GitHub's **Report a vulnerability** button when available. Include the
OS, Bulky version or commit, impact, and a minimal reproduction. Redact
credentials, private arguments, paths and environment values.

Bulky runs commands with the caller's permissions. Its pool belongs to one
OS user. For a shared-state issue, identify the owner of the files and which
other account can access or change them.

During development, report findings against the current source. Once 0.x
releases are available, the latest release receives security fixes.
