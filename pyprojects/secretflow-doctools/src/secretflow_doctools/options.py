from pydantic import BaseSettings


class DocToolsOptions(BaseSettings):
    class Config:
        env_prefix = "DOCTOOLS_"

    DOCKER_CLI_PATH: str = "docker"
    DOCKER_IMAGE: str = "secretflow/doctools:latest"
    DOCKER_IMAGE_PULL_LATEST: bool = False
