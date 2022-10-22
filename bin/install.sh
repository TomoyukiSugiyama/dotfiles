#!/bin/bash -ue

help() {
  command echo "Usage:"
  command echo "    $(basename ${0}) [--help | -h]" 0>&2
  command echo "Options:"
  command echo "    --help, -h        help message"
}

while [ $# -gt 0 ];do
  case ${1} in
    --help|-h)
      help
      exit 1
      ;;
    *)
      ;;
  esac
  shift
done