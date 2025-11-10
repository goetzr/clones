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
		- Pulls image from Docker Hub if not found locally
		- Regularly update your base images to benefit from security patches and bug fixes
			- Update by rebuilding the image, or
			- docker pull {image name}
	- WORKDIR: Specify the container application's working directory
	- COPY: Copy files from the host to the container
	- RUN: Run a command inside the container
	- EXPOSE: Expose a network port from the container
	- CMD: Set the default command to run inside the container
		- Without a command the container would start but immediately exit
		- Every container should run a single application
- Each directive corresponds to a layer in the resulting image
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
- Stored in a dedicated area on the host filesystem
- Mount volume:
	- docker run -v my-volume:/app/data my-image
	- Maps the my-volume volume at /app/data inside the container
	- Data written to /app/data inside the container is saved in my-volume and persists after the container is restarted/removed
## Bind Mounts
- Allow you to map a file or directory from the host directly into a container
- Useful to share source code with a container
## Networks
- Types of docker networks:
	- Bridge: The default network drive. Use this when different containers communicate with the same docker host.
	- Host: Use when you don't need any isolation between the container and the host.
	- None: Disables networking for the container.
	- macvlan: Assigns a unique MAC address to the container, making it appear as a physical device on the network.
# Internals

## docker
- Command line application that acts as a client, sending commands to dockerd
- Sends REST API requests
- UNIX socket or network socket
## dockerd
- Daemon process that acts as a server, processing commands received from docker
- Listens for REST API requests
- UNIX socket or network socket
- Hands the image and run configuration over to containerd when running a container
## containerd
- Manages container lifecycle (start, stop, pause, delete, etc.)
- Manages volumes
- Uses runc to create a new container
- Handles pulling container images from Docker Hub
- Manages container execution
- Manages container storage
- Provides a higher level API that abstracts away lower level details of container operations
- Acts as an intermediary between dockerd and runc
## Shim
- Exists between containerd and runc
- Enables daemonless containers
## runc
- Container runtime (interface to kernel primitives)
- Interfaces with the Linux kernel to create isolated namespaces and limit resources with cgroups
- Creates and runs containers
- Responsible for creating and managing containers based on the specifications defined in the container image (e.g. the filesystem)
- Focuses on the execution details of each container
- Creates cgroup
- Sets up the filesystem:
	- Mounts an overlay filesystem
	- Binds necessary host paths into the container's rootfs
	- Mounts pseudo filesystems like /proc and /sys
	- Finally performs pivot_root() to make the container's rootfs the actual /
	- After that, the original root is unmounted and detached
## Layered Filesystem
- Filesystem is broken down into different layers so that the same layer can be used for multiple images
- Achieved using OverlayFS
- Each layer is compressed
- A JSON metadata file of an image states the order of the layers to create a complete filesystem
### OverlayFS
- Union filesystem
- Start with a base layer (directories) on top of which a new layer is added
- Lower layers are usually read-only and upper layers are editable
- A container sees a full Linux filesystem, but it's a synthetic view built from layered images

# Kernel Primitives
## Inspecting Namespaces
- Find the container's main process PID:
	- docker inspect {container name} | grep -i pid
- List the namespaces for this process:
	- docker exec {container name} ls -la /proc/1/ns/
## Cgroup Namespace
- Control groups (cgroups) allows processes to be organized into hierarchical groups whose usage of various types of resources (CPU, memory, disk I/O, network I/O) can then be limited and monitored
- Create a cgroup by creating a directory inside /sys/fs/cgroup
	- mkdir /sys/fs/cgroup/my_cgroup
- The /sys folder is a virtual filesystem. Content inside it is not stored on disk, but contains information about the system and hardware configuration.
- To set a memory limit for the cgroup write it to the memory.max file:
	- echo {mem limit in bytes} > /sys/fs/cgroup/my_cgroup/memory.max
- To add a process with the PID 1234 for example to the cgroup, write the PID to the cgroup.procs file:
	- echo 1234 > /sys/fs/cgroup/my_cgroup/cgroup.procs
- Limit container to 50% of one CPU core:
	- docker run --cpus="0.5" {image name}
- Set CPU priority (relative weight):
	- docker run --cpu-shares=512 {low-priority-container}
	- docker run --cpu-shares=1024 {high-priority-container}
- Hard memory limit: container is killed if it exceeds 512 MB:
	- docker run --memory="512m" {image name}
- Soft limit: container stays under 256 MB unless memory is available:
	- docker run --memory="512m" --memory-reservation="256m" {image name}
- Disable swap usage entirely:
	- docker run --memory="512m" --memory-swap="512m" {image name}
- What happens when limits are exceeded:
	- CPU: Container is throttled (runs slower)
	- Memory: Container is killed with an OOM error (exit status 137)
		- docker logs my-app
- Limit read/write throughput to 1 MB/s:
	- docker run --device-read-bps /dev/sda:1mb {image name}
	- docker run --device-write-bps /dev/sda:1mb {image name}
- 
## User Namespace
- Isolates users and groups in the container from those on the host
- Enables containers to map user IDs inside the container to different user IDs on the host, improving security
	- 
## Mount Namespace
- Isolates filesystems in the container from those on the host
- Ensures that each container has its own separate filesystem and can mount file systems independently
## Network Namespace
- Isolates network interfaces and sockets from those on the host
- Allows each container to have its own network stack (interfaces, IP addresses, routes, etc.)
## PID Namespace
- Isolates processes running in the container from those running on the host
- Processes within a PID namespace only see processes in the same PID namespace
- Each PID namespace has its own numbering
- On the host Linux system, PID 1 is reserved for the main system process. Inside a container its main process is PID 1. However, on the host, the container's main process has a high-numbered PID.
## IPC Namespace
- Isolates IPC primitives (pipes, shared memory, message queues, semaphores, etc.) in the container from those on the host
## UTS (Unix Time Sharing) Namespace
- Isolates the hostname of the container from the hostname of the host
- Allows containers to have their own hostname and domain name
- docker run --hostname my-hostname {image name}
## Time Namespace
- Isolates the system time inside the container from the system time of the host

# Command Line Tools
## docker build
- Options:
	- -t: Sets the tag of the generated image
		- You can version containers by including a version number after the tag separated by a colon:
			- docker build -t my-ubuntu-app:v2 .
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



