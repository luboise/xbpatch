#!/bin/bash

act -P ubuntu-latest=ghcr.io/catthehacker/ubuntu:rust-latest --matrix os:ubuntu-latest && 

# Keep these separated, act can get confused when running these at the same time
act -P windows-latest=mcr.microsoft.com/devcontainers/rust:latest --matrix os:windows-latest
