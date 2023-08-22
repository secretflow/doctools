DOCKER_USERNAME ?= $(shell whoami)
APPLICATION_NAME ?= doctools

GIT_HASH ?= $(shell git log --format="%h" -n 1)

image:
	docker buildx build \
		--platform linux/amd64 \
		-t ${DOCKER_USERNAME}/${APPLICATION_NAME}:${GIT_HASH} \
		-f ./docker/doctools.Dockerfile .
