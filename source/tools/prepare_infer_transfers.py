"""Prepare fresh public application checkouts at the transfer suite's exact pins."""

import argparse
import os
from pathlib import Path
import re

from check_infer_transfers import SCENARIOS, check_application
from infer_experiment import LoggedCommands, positive_timeout, replay_receipt, require


REPOSITORIES = {
    "fedimint": "https://github.com/fedimint/fedimint.git",
    "redb": "https://github.com/cberner/redb.git",
}


def application_specs():
    applications = {}
    for scenario, specification in SCENARIOS.items():
        checkout = specification["checkout"]
        commit = specification["commit"]
        require(checkout in REPOSITORIES, "unknown application checkout: " + checkout)
        require(isinstance(commit, str) and re.fullmatch(r"[0-9a-f]{40}", commit),
                "application pin must be an exact commit")
        previous = applications.get(checkout)
        require(previous is None or previous["commit"] == commit,
                "conflicting pins for shared checkout: " + checkout)
        applications.setdefault(checkout, dict(scenario=scenario, commit=commit,
                                               repository=REPOSITORIES[checkout]))
    require(applications, "no application transfers are registered")
    return applications


def git_environment():
    environment = {name: value for name, value in os.environ.items()
                   if not name.startswith("GIT_")}
    environment.update(GIT_TERMINAL_PROMPT="0", GIT_CONFIG_GLOBAL=os.devnull,
                       GIT_CONFIG_NOSYSTEM="1")
    return environment


def check_repository_root(run, checkout, application, environment, phase):
    label = checkout + "_" + phase + "_root"
    require(run(label, ["git", "-C", str(application), "rev-parse", "--show-toplevel"],
                environment) == 0, "cannot inspect application repository root")
    actual = (run.output / (label + ".stdout")).read_text().strip()
    metadata = application / ".git"
    require(actual and Path(actual).resolve() == application.resolve()
            and metadata.is_dir() and not metadata.is_symlink(),
            "application repository root differs from its fresh destination")


def prepare(destination, output, timeout):
    destination = Path(destination)
    output = Path(output)
    require(not output.resolve().is_relative_to(destination.resolve())
            and not destination.resolve().is_relative_to(output.resolve()),
            "application and evidence destinations must be disjoint")
    require(not output.exists() and not output.is_symlink(),
            "evidence destination must not already exist")
    output.mkdir(parents=True, mode=0o700)
    receipt = dict(schema_version=1, kind="application_preparation", commands=[],
                   applications={}, application_proofs_proved=False)
    try:
        with replay_receipt(output, receipt):
            specifications = application_specs()
            require(not destination.exists() and not destination.is_symlink(),
                    "application destination must not already exist")
            destination.mkdir(parents=True, mode=0o700)
            environment = git_environment()
            run = LoggedCommands(output, destination, receipt["commands"], timeout=timeout)
            for checkout, specification in sorted(specifications.items()):
                application = destination.resolve() / checkout
                commands = [
                    ("init", ["git", "init", str(application)]),
                    ("remote", ["git", "-C", str(application), "remote", "add", "origin",
                                specification["repository"]]),
                    ("fetch", ["git", "-C", str(application), "fetch", "--no-tags",
                               "--depth=1", "origin", specification["commit"]]),
                    ("checkout", ["git", "-C", str(application), "checkout", "--detach",
                                  specification["commit"]]),
                ]
                for phase, command in commands:
                    require(run(checkout + "_" + phase, command, environment) == 0,
                            "cannot prepare application: " + checkout + " " + phase)
                    if phase == "init":
                        check_repository_root(run, checkout, application,
                                              environment, "initialized")
                check_repository_root(run, checkout, application, environment, "prepared")
                check_application(run, specification["scenario"], application,
                                  environment, "prepared")
                receipt["applications"][checkout] = dict(specification, path=str(application))
    finally:
        print(output)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=positive_timeout, default=300)
    arguments = parser.parse_args()
    prepare(arguments.destination, arguments.output, arguments.timeout)


if __name__ == "__main__":
    main()
