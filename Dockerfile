FROM scratch
RUN cp my.config /
CMD docker build -t seanmceligot/rusttest
