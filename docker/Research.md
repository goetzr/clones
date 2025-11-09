# General

# Components
## Dockerfile
- Describes the steps to create an image
	- Specify base image to use
	- Install dependencies
	- etc.
- All instructions run from top to bottom
- "Source code" of the image
- Run docker build on Dockerfile to generate image
- Directives:
	- FROM: Specify base image
	- WORKDIR: Specify the container application's working directory
	- COPY: Copy files from the host to the container
	- RUN: Run a command inside the container
	- EXPOSE: Expose a network port from the container
	- CMD: Set the default command to run inside the container
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
## Volume
- Containers store data temporarily. Once you restart or delete them, the data is gone.
- A volume is a way to store data outside the container so it persists even if the container is removed or restarted
- Mount volume:
	- docker run -v my-volume:/app/data my-image
	- Maps the my-volume volume at /app/data inside the container
	- Data written to /app/data inside the container is saved in my-volume and persists after the container is restarted/removed
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
## Layered Filesystem
- Filesystem is broken down into different layers so that the same layer can be used for multiple images
- Achieved using OverlayFS
- Each layer is compressed
- A JSON metadata file of an image states the order of the layers to create a complete filesystem
### OverlayFS
- Union filesystem
- Start with a base layer (directories) on top of which a new layer is added
- Lower layers are usually read-only and upper layers are editable

# Kernel Primitives
## Cgroup Namespace
- Control groups (cgroups) allows processes to be organized into hierarchical groups whose usage of various types of resources (CPU, memory, disk I/O, network I/O) can then be limited and monitored
## User Namespace
- Isolates users and groups in the container from those on the host
## Mount Namespace
- Isolates filesystems in the container from those on the host
## Network Namespace
- Isolates network interfaces and sockets from those on the host
## PID Namespace
- Isolates processes running in the container from those running on the host
## IPC Namespace
- Isolates IPC primitives (pipes, shared memory, etc.) in the container from those on the host
## UTS (Unix Time Sharing) Namespace
- Isolates the hostname of the container from the hostname of the host
## Time Namespace
- Isolates the system time inside the container from the system time of the host

# Command Line Tools
## docker build
- Options:
	- -t: Name the generated image
## docker run
- Creates then runs a new container from an image
- Options:
	- -d: Detached mode: Runs containers in the background.
	- -it: Interactive mode: Work directly inside a container.
	- -p: Port mapping: Maps an exposed port in a container to a port on your host.
	- --name: Gives the container a non-default name.
	- -v: Attach a volume to the container?
	- --rm: When the container stops, automatically delete it.
	- -e: Pass in environment variables (i.e. database passwords or API keys).
		- Alternatively, pass in environment files container all environment variables with --env option
- Docker runs containers in 2 ways:
	- -it: Interactive mode
		- Work directly inside a container using the terminal
		- -i: Keeps the input open so you can type commands into the container
		- -t: Gives you a terminal-like interface inside the container
	- -d: Detached mode
		- - Runs the container in the background so your terminal is free for other tasks
## docker ps
- Lists all active containers running on your system
- Options:
	- -a: Include all containers, such as those that exited or were created but not started
	- -l (last): View only the most recently created container
## docker start
## docker stop
- Stop a specific running container:
	- docker stop {container ID or name}
- Stop all containers:
	- docker stop $(docker ps -q)
## docker rm
- Once a container is stopped you can remove it:
	- docker rm {container ID or name}
- Remove all stopped containers:
	- docker container prune
## docker exec
- Lets you run a new command inside an existing running container without stopping or restarting it
- One use is to open a new terminal inside the container to poke around, run scripts, check logs, or debug
## docker attach
- Connects your terminal directly to a running container's main process
- Allows you to see live log output, prompt, etc., and even interact with it if the process accepts input
## docker images



