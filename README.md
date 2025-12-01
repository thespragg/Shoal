
<p align="center">
  <img src="https://github.com/user-attachments/assets/0675c3a6-a7d4-4886-aced-fec95db518b9" alt="Shoal illustration" width="200" />
</p>

<h1 align="center">Shoal</h1>
<p align="center">
  <a href="https://github.com/thespragg/shoal/actions"><img src="https://img.shields.io/github/actions/workflow/status/thespragg/shoal/ci.yml?branch=main" alt="build status"></a>
  <img src="https://img.shields.io/badge/license-AGPL-blue.svg" alt="license">
</p>

## What is Shoal

Shoal is a simple tool to define, manage, and run multi-service stacks using Docker Compose. Including support for dependencies, local or remote service sources, and per-environment or per-developer overrides. 

## Who is Shoal for

Shoal was built for teams working with many services, with frequent context-switching, and a need to spin up small, consistent subsets of the system quickly. Shoal supports fast onboarding with org level services and stacks, by automating local orchestration while remaining flexible enough for custom overrides and service-specific workflows to fit each developer's specific need. In future Shoal will support stubbing and seeding.

## Shoal vs Docker Compose

### When to use Shoal
- You have **many microservices** (10+) and need to run different subsets frequently
- Developers need **different configurations** of the same stack
- You want **organization-wide stack definitions** that developers can customize locally
- You need **automatic dependency resolution** for complex service graphs
- Your team frequently **switches between feature contexts** requiring different service combinations

### When to use Docker Compose directly
- You have a **simple stack** (5 or fewer services) with minimal variation
- All developers run the **same configuration** consistently
- You prefer **explicit control** over every service in your compose file
- You're comfortable using [compose file merging](https://docs.docker.com/compose/how-tos/multiple-compose-files/merge/) for variations

**TLDR**: If most of your stacks share the same services with little variance, regular Docker Compose with merging might suit your use case better. Shoal shines when managing complexity across many services and configurations.

## Features

- Define services that use local repos or docker images
- Build stacks with dependent services
- Manage environment variables, internal ports, and custom Dockerfiles  
- Apply overrides for custom setups
- Easy CLI-based stack management
- Docker & Docker-compose under the hood, easy to debug and use the power of docker once the stack is running
- (Coming soon) Data seeding
- (Coming soon) Stub generation

## Installation

Download the binary from [Releases](https://github.com/thespragg/Shoal/releases). Package managers to be added later.

## Configuration

Configuration folders can be defined within `~/.shoal/` or within the folder Shoal is being run from. For a standard distributed org setup:

Create a folder to contain the stacks and services
```bash
mkdir orchestration && cd orchestration
```

Create the required folders and start by defining your services. Services can be defined using the condensed Shoal syntax shown below, or you can use a full Docker Compose service definition for complete control.
```bash
mkdir services && mkdir stacks && mkdir overrides
```

### Service Definitions
```yaml
# ./services/shoal-frontend.yml
name: demo-frontend
source:
  type: Image # can be Local
  image: nginxdemos/hello:latest # When running from local `path` must be defined.
dependencies:
  - demo-backend-1
  - demo-backend-2
internal_ports:
  - 80

# ./services/shoal-backend-1.yml
name: demo-backend-1
source:
  type: Image
  image: httpd:latest
dependencies:
  - postgres
  - subscription-provider
internal_ports:
  - 5000

# ./services/shoal-backend-2.yml
name: demo-backend-2
source:
  type: Image
  image: httpd:latest
dependencies:
  - redis
internal_ports:
  - 5000
  
# ./services/subscription-provider.yml
name: subscription-provider
source: 
  type: Image
  image: httpd:latest
dependencies:
  - postgres
internal_ports:
  - 5000
     
# ./services/redis.yml
name: redis
source:
  type: Image
  image: redis:latest
internal_ports:
  - 6379
    
# ./services/postgres.yml
name: postgres
source: 
  type: Image
  image: postgres:latest
env:
  POSTGRES_USER: postgres
  POSTGRES_PASSWORD: postgres
  POSTGRES_DB: postgres
internal_ports:
  - 5432
```

**Note**: For `source: local`, use `path: ./path/to/folder` pointing to a directory containing `Dockerfile.dev` or `Dockerfile.shoal`.

### Stack Definitions

Once the services have been defined, you can build stacks. Dependencies are loaded dynamically based on the dependency tree.
```yaml
# ./stacks/full-stack.yml
name: full-demo-stack
description: A stack containing all of the services
services:
  - demo-frontend # all dependencies will be satisfied
overrides:
  demo-backend-1:
    env:
      - ConnectionString=localhost:5432
      - LoggingLevel=Info
  shoal-backend-2:
    env:
      - RedisConnection=localhost
      - LoggingLevel=Info
            
# ./stacks/feature-stack-1.yml
name: demo-feature-stack-1
description: A stack with backend-2 excluded
services:
  - demo-frontend
exclude:
  - demo-backend-2 # demo-backend-2 and its dependencies will be ignored
overrides:
  demo-backend-1:
    env:
      - ConnectionString=localhost:5432
      - LoggingLevel=Info
```

At this point Shoal is ready to use! 

**Run a stack:**
```bash
shoal up full-stack
# or
shoal up feature-stack-1
```

**Output compose file without running:**
```bash
shoal up full-stack -o path/to/save/location/docker-compose.yml
```

### Overrides
 
Overriding is a core part of Shoal, giving developers the ability to save configurations that suit their needs. For example, if you want to log at a trace level:
```yaml
# ./overrides/trace-logging.yml
name: trace-logging
stack: full-stack
description: Sets all service log levels to trace

overrides:
  demo-backend-1:
    env:
      LoggingLevel: Trace
  demo-backend-2:
    env:
      LoggingLevel: Trace
```

**Apply an override:**
```bash
shoal up full-stack.trace-logging
```

Overrides can be stored in the repo for shared configurations, or in `~/.shoal/overrides` for developer-specific ones. All service fields can be overridden, whether you need to mount extra volumes, expose more ports, or change any other configuration.

## Todo
- Most of this readme...