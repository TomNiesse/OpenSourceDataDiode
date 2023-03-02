FROM alpine
ARG file
ENV env_file=$file
COPY $env_file .
# Create a directory for ph_fuse in all docker containers,
# since there seems no way to do this in a better way.
RUN mkdir /fusemount
ENTRYPOINT ./$env_file