# Domain Pitfalls

**Domain:** Docker-based Database Management Tool (LDB-Engine)
**Researched:** 2026-02-27
**Overall Confidence:** HIGH

## Executive Summary

This document catalogs critical pitfalls specific to building a Docker-based local database management tool on Linux. The research covers Docker socket permissions, volume management, port conflicts, and log streaming—four areas where database management tools commonly fail. Each pitfall includes warning signs, prevention strategies, and phase mapping for the LDB-Engine roadmap.

---

## Critical Pitfalls

Mistakes that cause data loss, security vulnerabilities, or complete feature failure.

### Pitfall 1: Docker Socket Permission Denied on Linux

**What Goes Wrong:** Users cannot access the Docker daemon because they lack permissions to `/var/run/docker.sock`. The application fails to start containers with a cryptic "permission denied" error.

**Why It Happens:** 
- Docker socket (`/var/run/docker.sock`) is owned by `root:docker` with 660 permissions
- Users not in the `docker` group cannot access the socket
- The application runs under the current user's context without elevated privileges
- Some Linux distributions (newer Debian, Fedora) no longer auto-create the docker group

**Consequences:**
- Application cannot create or manage any containers
- Complete feature failure—core value proposition broken
- Poor user experience: cryptic error messages confuse users

**Warning Signs:**
- First-run failures on fresh Linux installations
- Users who installed Docker via snap or non-standard methods
- Error message: `permission denied while trying to connect to the Docker daemon socket at unix:///var/run/docker.sock`

**Prevention Strategy:**
1. **Detection at startup:** Check Docker socket accessibility on app launch
2. **Provide actionable guidance:** Display clear instructions when socket is inaccessible:
   - Check if user is in docker group: `groups $USER`
   - Provide one-click "Add to Docker Group" button with sudo prompt
   - Fallback: Display manual instructions if auto-fix fails
3. **Handle edge cases:**
   - Docker installed via snap (different socket path: `~/snap/docker/current/docker.sock`)
   - Rootless Docker mode (different socket path)
   - Docker not installed at all

**Phase to Address:** Phase 2 (Instance Creation & Configuration) — requires Docker daemon access to function

**Confidence:** HIGH — well-documented Linux issue with known solutions

---

### Pitfall 2: Anonymous Volume Data Loss on Container Recreation

**What Goes Wrong:** Database data disappears when containers are recreated or upgraded, even though volumes appeared to work.

**Why It Happens:**
- Using `-v /var/lib/postgresql/data` (anonymous volume) instead of named volume
- When container is removed and recreated, Docker cannot reconnect to the original anonymous volume
- Each recreation creates a fresh empty volume
- Common mistake when using `docker run` without explicit volume names

**Consequences:**
- Complete data loss for user's database
- Catastrophic for a database management tool—trust destroyed
- Particularly dangerous during "upgrade" operations

**Warning Signs:**
- Database empty after container restart
- Data disappears after upgrading database image version
- `docker volume ls` shows many anonymous volumes (starting with long hash)

**Prevention Strategy:**
1. **Always use named volumes:** Enforce `docker run -v db_data:/var/lib/postgresql/data` pattern
2. **Validate volume existence:** Before starting container, verify named volume exists
3. **Backup before operations:** Create volume snapshot before upgrade/recreate operations
4. **UI enforcement:** Never expose anonymous volume option to users

**Phase to Address:** Phase 4 (Persistent Data Management) — volume management is core to this phase

**Confidence:** HIGH — well-documented Docker behavior, critical for database tools

---

### Pitfall 3: Port Conflict Failures

**What Goes Wrong:** Container fails to start with "bind: address already in use" error because requested port is occupied.

**Why It Happens:**
- User tries to create PostgreSQL on port 5432, but native PostgreSQL is running
- Previous container was killed but port still in TIME_WAIT state
- Multiple database instances requested on same port
- Docker Compose stack left orphaned port bindings

**Consequences:**
- User cannot start database instance
- Confusing error message doesn't identify the conflicting process
- Trial-and-error frustration

**Warning Signs:**
- "Error starting userland proxy: listen tcp 0.0.0.0:5432: bind: address already in use"
- Container exits immediately after start
- Port scan shows port occupied but `docker ps` shows no container

**Prevention Strategy:**
1. **Port availability check:** Before binding, verify port is available:
   ```bash
   # Check if port is in use
   lsof -i :5432
   netstat -tulpn | grep :5432
   ```
2. **Smart port assignment:**
   - Default to common ports (5432, 3306, 27017, 6379)
   - If occupied, auto-increment (5433, 5434, etc.)
   - Allow user override with clear warning
3. **Display port ownership:** When conflict detected, show what's using the port (process name, PID)
4. **Handle TIME_WAIT:** Wait or use `SO_REUSEADDR` option where possible

**Phase to Address:** Phase 2 (Instance Creation & Configuration) — port configuration happens at creation

**Confidence:** HIGH — extremely common issue, well-documented solutions

---

### Pitfall 4: Log Streaming Performance Degradation

**What Goes Wrong:** Real-time log viewer becomes unresponsive, consumes excessive memory, or drops connections when containers produce high log volume.

**Why It Happens:**
- Using `docker logs -f` without limits on containers with verbose logging
- No buffering—every log line triggers UI update
- JSON log driver creates large files consumed entirely
- No backpressure handling when log production exceeds consumption

**Consequences:**
- UI freezes or becomes unusable
- Memory exhaustion from accumulated log data
- Application becomes unresponsive
- User cannot debug their database issues

**Warning Signs:**
- Log viewer becomes sluggish after running for several minutes
- Memory usage grows continuously
- "Connection reset" or "stream interrupted" errors
- High CPU usage from constant log polling

**Prevention Strategy:**
1. **Limit log tail:** Always use `--tail` flag to limit buffered logs
   ```bash
   docker logs --tail 1000 -f container_id
   ```
2. **Implement virtual scrolling:** Only render visible log lines, not entire buffer
3. **Add pause/resume:** Allow users to pause auto-scroll during high-volume output
4. **Consider alternative log access:**
   - Direct file access to container log: `/var/lib/docker/containers/<id>/<id>-json.log`
   - Use logging driver configuration: `--log-opt max-size=10m --log-opt max-file=3`
5. **Stream management:** Limit number of concurrent log streams (max 5-10 containers)

**Phase to Address:** Phase 6 (Real-time Log Viewer) — performance is core to this feature

**Confidence:** MEDIUM — performance issues depend on usage patterns, mitigation strategies are known

---

## Moderate Pitfalls

Mistakes that cause delays, poor UX, or significant technical debt.

### Pitfall 5: Image Pull Failures and Version Drift

**What Goes Wrong:** Database image pull fails, or pulls unexpected version causing compatibility issues.

**Why It Happens:**
- Using `:latest` tag—image changes underneath without notice
- Network issues during pull (especially large database images 500MB+)
- Rate limiting from Docker Hub (anonymous pulls limited)
- Corrupted image layers from interrupted pulls

**Consequences:**
- "1-click" promise broken—manual intervention required
- Unexpected behavior after system update
- Pull failures frustrate new users

**Warning Signs:**
- `docker pull` hangs or times out
- "manifest unknown" errors for architecture mismatches
- Image works on one machine, fails on another

**Prevention Strategy:**
1. **Pin specific versions:** Use explicit version tags (e.g., `postgres:15.4`, not `postgres:latest`)
2. **Provide version selector:** Let users choose from curated list of stable versions
3. **Pre-cache popular images:** On first run, offer to pull common database images in background
4. **Handle pull failures gracefully:**
   - Retry with exponential backoff
   - Show clear progress indication
   - Provide offline mode if image exists locally
5. **Document Docker Hub limits:** Warn users about rate limits, suggest authenticated pulls

**Phase to Address:** Phase 1 (Docker Hub Integration & Image Discovery) — version management is part of discovery

**Confidence:** HIGH — well-known Docker Hub limitations, standard solutions exist

---

### Pitfall 6: Container Cleanup Debt

**What Goes Wrong:** Stopped containers, unused volumes, and dangling images accumulate over time, consuming disk space.

**Why It Happens:**
- No automatic cleanup policy
- `docker rm` doesn't remove volumes by default
- Anonymous volumes accumulate from failed/recreated containers
- Log files grow unbounded

**Consequences:**
- Disk space exhaustion after extended use
- System performance degradation
- Users blame the application for disk issues

**Warning Signs:**
- `docker system df` shows high space usage
- Volume count grows without bound
- Log files in `/var/lib/docker/containers/` exceed several GB

**Prevention Strategy:**
1. **Expose cleanup UI:** "Clean up unused resources" button
2. **Auto-cleanup option:** Settings to auto-remove stopped containers after X days
3. **Volume management UI:** List volumes with sizes, allow deletion
4. **Log rotation:** Configure log max-size and max-file:
   ```bash
   docker run --log-opt max-size=10m --log-opt max-file=3 ...
   ```
5. **Disk space warnings:** Monitor and alert when disk space low

**Phase to Address:** Phase 3 (Instance Lifecycle Management) — cleanup is part of lifecycle

**Confidence:** HIGH — common Docker maintenance issue

---

### Pitfall 7: Insecure Container Configuration

**What Goes Wrong:** Database containers run with excessive privileges, exposing security vulnerabilities.

**Why It Happens:**
- Running with `--privileged` flag for convenience
- Not specifying security options
- Binding to all interfaces `0.0.0.0` instead of localhost
- Using default credentials

**Consequences:**
- Security vulnerability if machine is compromised
- Container escape possible
- Data exposure risk

**Warning Signs:**
- Security scanner warnings
- Container can access host resources it shouldn't

**Prevention Strategy:**
1. **Security defaults:**
   ```bash
   # Don't use --privileged
   # Use --network bridge (default)
   # Bind to 127.0.0.1 only, not 0.0.0.0
   docker run -p 127.0.0.1:5432:5432 ...
   ```
2. **Read-only root filesystem:** `--read-only=true` where possible
3. **Drop capabilities:** `--cap-drop=ALL --cap-add=NET_BIND_SERVICE`
4. **User namespace remapping:** Consider `--userns-remap` for isolation
5. **No default passwords:** Force password change on first use

**Phase to Address:** Phase 2 (Instance Creation & Configuration) — security config happens at creation

**Confidence:** MEDIUM — security best practices are documented but specific to use case

---

### Pitfall 8: Container Health Monitoring Gaps

**What Goes Wrong:** Application shows database as "running" but it's actually unhealthy or stuck.

**Why It Happens:**
- Only checking container status (`docker ps`), not health
- Database process inside container crashed but container still running
- No health check configured
- Long-running queries causing apparent hangs

**Consequences:**
- User thinks database is available but connections fail
- Attempts to connect to unhealthy database cause confusion
- Silent data issues

**Warning Signs:**
- Container "running" but `docker logs` shows repeated crash/restart
- Connection timeouts despite container status
- Health check configured but not working as expected

**Prevention Strategy:**
1. **Configure health checks:** Use Docker HEALTHCHECK instruction
   ```bash
   docker run --health-cmd="pg_isready -U postgres" \
              --health-interval=30s \
              --health-timeout=10s \
              --health-retries=3 ...
   ```
2. **Display health status:** Show health in UI (healthy/unhealthy/starting)
3. **Auto-restart on failure:** Use `--restart=unless-stopped` for critical databases
4. **Connection testing:** Actually test connection before showing "ready"

**Phase to Address:** Phase 3 (Instance Lifecycle Management) — health monitoring is lifecycle

**Confidence:** MEDIUM — health check configuration is standard but database-specific

---

## Minor Pitfalls

Mistakes that cause annoyance but are easily fixable.

### Pitfall 9: Missing Container Identification

**What Goes Wrong:** Users cannot distinguish between multiple database containers in the UI.

**Why It Happens:**
- Containers only identified by random IDs in Docker
- No metadata/label convention used
- Generic names like "postgres-1", "postgres-2"

**Consequences:**
- Confusion when managing multiple databases
- Wrong container stopped/deleted
- Poor UX

**Prevention Strategy:**
1. **Use container naming:** Allow user-defined names, enforce uniqueness
2. **Add labels:** Use Docker labels for metadata
   ```bash
   docker run --label "ldb.engine.name=my-production-db" ...
   ```
3. **Display database type:** Clear icon and name (PostgreSQL 15, MySQL 8, etc.)
4. **Show ports and volumes:** At-a-glance container identification

**Phase to Address:** Phase 3 (Instance Lifecycle Management) — identification is part of management UI

**Confidence:** HIGH — simple UX improvement, standard Docker feature

---

### Pitfall 10: No Connection String Generation

**What Goes Wrong:** Users struggle to construct correct connection strings for their applications.

**Why It Happens:**
- Connection strings have complex format
- Users don't know required parameters
- No way to copy/paste connection details

**Consequences:**
- Users cannot connect their apps to databases
- Support burden increases
- "Doesn't work" reviews

**Prevention Strategy:**
1. **Generate connection strings:** Provide ready-to-use formats
   - JDBC: `jdbc:postgresql://localhost:5432/mydb`
   - CLI: `psql -h localhost -p 5432 -U postgres -d mydb`
   - Environment variable: `DATABASE_URL=postgresql://...`
2. **One-click copy:** Copy button for each format
3. **Test connection:** Verify connectivity from within app
4. **Show all parameters:** Host, port, username, database, password (masked)

**Phase to Address:** Phase 5 (Connection Utilities) — connection string generation is core to this phase

**Confidence:** HIGH — standard feature expected in database tools

---

## Phase-Specific Warnings

| Phase | Critical Pitfalls to Address First | Notes |
|-------|-----------------------------------|-------|
| Phase 1: Docker Hub Integration | Pitfall 5 (Image Pull Failures) | Version pinning critical before discovery |
| Phase 2: Instance Creation | Pitfalls 1, 3, 7 | Permissions, ports, security must work |
| Phase 3: Lifecycle Management | Pitfalls 6, 8, 9 | Cleanup and health monitoring essential |
| Phase 4: Persistent Data | Pitfall 2 (Data Loss) | Most critical—data loss destroys trust |
| Phase 5: Connection Utilities | Pitfall 10 | Connection strings expected by users |
| Phase 6: Log Viewer | Pitfall 4 (Performance) | Performance critical for usability |

---

## Confidence Assessment

| Area | Confidence | Reason |
|------|------------|--------|
| Docker Socket Permissions | HIGH | Well-documented Linux issue, standard solutions |
| Volume Management | HIGH | Docker volume behavior is documented |
| Port Conflicts | HIGH | Extremely common, well-understood |
| Log Streaming Performance | MEDIUM | Performance depends on usage patterns |
| Database-Specific Issues | MEDIUM | General patterns known, database-specific needs vary |

---

## Sources

### High Confidence (Official Documentation)
- Docker Official Documentation: Linux post-install, protect access
- MySQL 8.4 Reference Manual: Deploying MySQL on Linux with Docker
- Docker Engine security documentation

### Medium Confidence (Verified Community Sources)
- Stack Overflow: Port binding conflicts (highly voted solutions)
- Docker community forums: Socket permission solutions
- Multiple blog posts on volume management best practices

### Low Confidence (Unverified, Flagged for Validation)
- Specific logging driver performance benchmarks
- Exact memory usage numbers for log streaming
- Security hardening specifics for database containers

---

## Gaps to Address

The following areas may need phase-specific research:

1. **Phase 4 deeper research:** Volume backup/restore strategies for each supported database
2. **Phase 6 deeper research:** WebSocket vs polling for log streaming performance
3. **Security audit:** Container security hardening specifics may need verification
