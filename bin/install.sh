#!/bin/bash -ue

help() {
  command echo "Usage: $0 [--help | -h]" 0>&2
  command echo ""
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