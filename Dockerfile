# syntax=docker/dockerfile:1
FROM node:24-bookworm-slim@sha256:01d566e8d94d8c619591d78652babfccd3c6992a3dffa9ed5ab39e11cf6c70e9
WORKDIR /workspace
RUN chown node:node /workspace
USER node
COPY --chown=node:node package.json package-lock.json ./
RUN npm ci --no-audit --no-fund
# Include only inputs needed by the frontend and static security checks.
COPY --chown=node:node index.html tsconfig.json vite.config.ts ./
COPY --chown=node:node src ./src
COPY --chown=node:node tests ./tests
COPY --chown=node:node src-tauri/tauri.conf.json ./src-tauri/tauri.conf.json
COPY --chown=node:node src-tauri/capabilities ./src-tauri/capabilities
COPY --chown=node:node scripts/verify-docker.sh ./scripts/verify-docker.sh
CMD ["sh", "scripts/verify-docker.sh"]
