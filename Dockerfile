FROM scratch
ARG TARGETARCH
ADD https://github.com/wheegee/sentential-entry/releases/latest/download/entry-${TARGETARCH} /entry
COPY ./wrapper.sh /wrapper.sh
