# Notifier

## Purpose

To give me a reminder to stretch/drink water etc based on what I configure.

## How it works

It loads a configuration file and parses it. It uses cron jobs to schedule notifications as a reminder.

### Configuration
The file is `config.yaml` and should be placed in `$HOME/.config/`
```YAML
---
notifications:
- label: Stretch
  cron: 0 0 7-15 * * 1-5 *
  level: "Info"
- label: Drink water
  cron: 0 5 7-15 * * 1-5 *
  level: "Info"

```

The cron strucutre is
```
sec   min   hour   day of month   month   day of week   year
*     *     *      *              *       *             *
```

More details about the cron structure can be found at https://crates.io/crates/job_scheduler

### Installation

Either run
`sh install.sh` or `./install.sh`
OR
`cargo install --path .`

This will install the executable locally on the system.
NOTE: This does not setup the executable to run at start up.

### Support

It is supported on Linux and Windows. MacOS still needs to be tested but should theoretically work.

### Start Up

A script or instructions to have the executable run at start up still needs to be added

## TODO

 * Exit more gracefully
 * Verbose mode
 * Write tests
