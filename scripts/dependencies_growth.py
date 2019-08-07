import toml

dependency_keys = ["dependencies", "dev-dependencies", "build-dependencies"]


def main():
    # dependency list
    dependencies = set()
    # parse workspace
    ff = open("Cargo.toml")
    workspace = toml.loads(ff.read())
    for member in workspace["workspace"]["members"]:
        # each cargo.toml
        ff = open(member + "/Cargo.toml")
        crate = toml.loads(ff.read())
        # dependencies
        for dependency_type in dependency_keys:
            if dependency_type in crate:
                for dependency, options in crate[dependency_type].items():
                    # ignore internal dependencies
                    if "path" not in options:
                        dependencies.add(dependency)
    # print dependencies
    for dependency in dependencies:
        print(dependency)


if __name__ == "__main__":
    main()
