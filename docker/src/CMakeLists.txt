cmake_minimum_required(VERSION 4.1)
#set(CMAKE_EXPERIMENTAL_CXX_IMPORT_STD d0edc3af-4c50-42ea-a356-e2862fe7a444)
project(my_containerd CXX)

# Specify the C++ standard to use.
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

find_package(CURL REQUIRED)

add_executable(${PROJECT_NAME} main.cpp)
#set_property(TARGET ${PROJECT_NAME} PROPERTY CXX_MODULE_STD ON)
#target_compile_options(${PROJECT_NAME} PRIVATE "-fmodules")
target_precompile_headers(${PROJECT_NAME} PRIVATE pch.h)
target_include_directories(${PROJECT_NAME} PRIVATE CURL_INCLUDE_DIRS)
target_link_libraries(${PROJECT_NAME} PRIVATE CURL::libcurl)