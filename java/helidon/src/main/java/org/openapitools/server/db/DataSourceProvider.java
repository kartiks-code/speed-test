package org.openapitools.server.db;

import com.zaxxer.hikari.HikariConfig;
import com.zaxxer.hikari.HikariDataSource;
import jakarta.annotation.PostConstruct;
import jakarta.annotation.PreDestroy;
import jakarta.enterprise.context.ApplicationScoped;
import jakarta.inject.Inject;
import org.eclipse.microprofile.config.inject.ConfigProperty;

import javax.sql.DataSource;

@ApplicationScoped
public class DataSourceProvider {

    @Inject
    @ConfigProperty(name = "POSTGRES_HOST", defaultValue = "localhost")
    private String host;

    @Inject
    @ConfigProperty(name = "POSTGRES_PORT", defaultValue = "5432")
    private int port;

    @Inject
    @ConfigProperty(name = "POSTGRES_DB", defaultValue = "java-helidon")
    private String dbName;

    @Inject
    @ConfigProperty(name = "POSTGRES_USER", defaultValue = "postgres")
    private String user;

    @Inject
    @ConfigProperty(name = "POSTGRES_PASSWORD", defaultValue = "mysecret")
    private String password;

    private HikariDataSource dataSource;

    @PostConstruct
    void init() {
        HikariConfig cfg = new HikariConfig();
        cfg.setJdbcUrl("jdbc:postgresql://" + host + ":" + port + "/" + dbName);
        cfg.setUsername(user);
        cfg.setPassword(password);
        cfg.setMaximumPoolSize(5);
        dataSource = new HikariDataSource(cfg);
    }

    @PreDestroy
    void close() {
        if (dataSource != null && !dataSource.isClosed()) {
            dataSource.close();
        }
    }

    public DataSource get() {
        return dataSource;
    }
}
