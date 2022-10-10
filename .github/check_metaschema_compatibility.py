import glob
import re
import semver
import subprocess

metaschema_file_version_pattern = "^metaschema/near-abi-(\d+\.\d+\.\d+)-schema\.json$"
rust_schema_version_pattern = "^pub const SCHEMA_VERSION: &str = \"(\d+\.\d+\.\d+)\";$"

persisted_semvers = []
for metaschema in glob.glob("metaschema/near-abi-*.*.*-schema.json"):
    persisted_semver = semver.VersionInfo.parse(
        re.match(metaschema_file_version_pattern, metaschema).group(1)
    )
    persisted_semvers.append(persisted_semver)
persisted_semvers.sort()
last_persisted_semver = persisted_semvers[-1]
print("Last persisted ABI schema version:", last_persisted_semver)

current_semver = None
with open("near-abi/src/lib.rs", "r") as sources:
    for line in sources.readlines():
        match = re.match(rust_schema_version_pattern, line)
        if match is not None:
            current_semver = semver.VersionInfo.parse(match.group(1))
            break
if current_semver is None:
    print("Could not parse the current ABI schema version. Have you changed the way SCHEMA_VERSION is exposed?")
    exit(1)
print("Current ABI schema version:", current_semver)

if (current_semver.major > last_persisted_semver.major or
        (current_semver.major == 0 and
         last_persisted_semver.major == 0 and
         current_semver.minor > last_persisted_semver.minor)):
    print("Current ABI schema is allowed to make breaking changes against ",
          last_persisted_semver)
    exit(0)
else:
    result = subprocess.run(
        ["jsonschemacompat",
         "metaschema/near-abi-" + str(last_persisted_semver) + "-schema.json",
         "metaschema/near-abi-current-schema.json"]
    )
    exit(result.returncode)
