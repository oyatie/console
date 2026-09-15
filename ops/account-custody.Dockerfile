FROM postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280
COPY ops/postgres-finalize-account-custody.sh /opt/console-account-custody/
COPY ops/postgres-finalize-account-custody.sql /opt/console-account-custody/
COPY ops/postgres-finalize-account-credentials.sql /opt/console-account-custody/
COPY ops/postgres-install-durability-observer.sql /opt/console-account-custody/
COPY ops/account-custody-migrations.sha384 /opt/console-account-custody/
ENTRYPOINT ["bash", "/opt/console-account-custody/postgres-finalize-account-custody.sh"]
