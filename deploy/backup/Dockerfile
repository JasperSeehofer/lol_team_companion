# Backup sidecar for compose.yaml: rclone + docker CLI + tar/zstd, runs
# scripts/backup.sh once a day at BACKUP_AT (UTC). Build context: repo root.
FROM rclone/rclone:1.75.2

# rclone/rclone is Alpine based. coreutils: GNU date (-d) and du -b;
# docker-cli: stop/start the app around the copy; util-linux: flock.
RUN apk add --no-cache bash coreutils findutils grep tar zstd gzip curl docker-cli util-linux tzdata

COPY scripts/backup.sh /usr/local/bin/backup.sh
COPY deploy/backup/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod 0755 /usr/local/bin/backup.sh /usr/local/bin/entrypoint.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["schedule"]
