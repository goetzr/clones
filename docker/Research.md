# Components
## Dockerfile
- Describes the steps to create an image
	- Specify base image to use
	- Install dependencies
	- etc.
- All instructions run from top to bottom
- "Source code" of the image
- Run docker build on Dockerfile to generate image
## Image
- Read-only template used to create containers (static, read-only)
- Created from Dockerfile using docker build
- Immutable. Can't change until it is rebuilt.
- Made up of multiple layers
- Contains everything needed to run an application (application, libraries, environment variables, configuration and data files, etc.)
- Stored on disk
## Container
- Runnable instance of an image (dynamic, executable)
- Runs in an isolated process on the host
- Shares the host kernel
- Created when a docker image is run
- Mutable. Can modify it while it's running, install new packages, etc.
- Runs in memory
# Internals

## docker
- Command line application that acts as a client, sending commands to dockerd
- Sends REST API requests
- UNIX socket or network socket
## dockerd
- Daemon process that acts as a server, processing commands received from docker
- Listens for REST API requests
- UNIX socket or network socket
## containerd
- Manages container lifecycle (start, stop, pause, delete, etc.)
- Manages volumes???
## Shim
- Exists between containerd and runc
- Enables daemonless containers
## runc
- Container runtime (interface to kernel primitives)

# Command Line Tools
## docker build
## docker run
- Creates then runs a new container from an image
- Flags:
	- -d: Detached mode: Runs containers in the background.
	- -p: Port mapping: Maps an exposed port in a container to a port on your host.
	- --name: Gives the container a non-default name.
	- -v: Attach a volume to the container?
	- --rm: When the container stops, automatically delete it.
	- -e: Pass in database passwords or API key without hard-coding them in your app.
## docker container
## docker ps
## docker start
## docker stop
## docker exec



