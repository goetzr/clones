cmake_minimum_required(VERSION 3.28)
project(test VERSION 0.0.1 LANGUAGES C)

add_executable(test test.c wireguard.c)
