package petstore

import (
	"context"
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"net/url"
	"os"
	"strconv"
	"strings"
	"time"

	_ "github.com/jackc/pgx/v5/stdlib"
)

var (
	ErrNotFound     = errors.New("not found")
	ErrInvalidInput = errors.New("invalid input")
)

type Store interface {
	Close() error
	CreatePet(context.Context, Pet) (Pet, error)
	UpdatePet(context.Context, Pet) (Pet, error)
	GetPetByID(context.Context, int64) (Pet, error)
	DeletePet(context.Context, int64) (bool, error)
	FindPetsByStatus(context.Context, []string) ([]Pet, error)
	FindPetsByTags(context.Context, []string) ([]Pet, error)
	UpdatePetFields(context.Context, int64, *string, *string) (bool, error)
	SavePetPhoto(context.Context, int64, []byte, string, string) error
	Inventory(context.Context) (map[string]int32, error)
	CreateOrder(context.Context, Order) (Order, error)
	GetOrderByID(context.Context, int64) (Order, error)
	DeleteOrder(context.Context, int64) (bool, error)
	CreateUser(context.Context, User) (User, error)
	CreateUsers(context.Context, []User) (User, error)
	GetUserByUsername(context.Context, string) (User, error)
	UpdateUser(context.Context, string, User) (bool, error)
	DeleteUser(context.Context, string) (bool, error)
	AuthenticateUser(context.Context, string, string) (bool, error)
}

type PostgresStore struct {
	db *sql.DB
}

func NewPostgresStore(ctx context.Context, dsn string) (*PostgresStore, error) {
	db, err := sql.Open("pgx", dsn)
	if err != nil {
		return nil, err
	}
	applyPoolSettingsFromEnv(db)
	if err := db.PingContext(ctx); err != nil {
		db.Close()
		return nil, err
	}
	return &PostgresStore{db: db}, nil
}

// applyPoolSettingsFromEnv tunes the connection pool from optional env vars.
// Unset (or unparsable) variables leave the database/sql defaults untouched.
func applyPoolSettingsFromEnv(db *sql.DB) {
	if v, err := strconv.Atoi(os.Getenv("DB_MAX_OPEN_CONNS")); err == nil {
		db.SetMaxOpenConns(v)
	}
	if v, err := strconv.Atoi(os.Getenv("DB_MAX_IDLE_CONNS")); err == nil {
		db.SetMaxIdleConns(v)
	}
	if v, err := strconv.Atoi(os.Getenv("DB_CONN_MAX_IDLE_TIME_SECONDS")); err == nil {
		db.SetConnMaxIdleTime(time.Duration(v) * time.Second)
	}
}

func NewPostgresStoreFromEnv(ctx context.Context) (*PostgresStore, error) {
	dsn, err := postgresDSNFromEnv()
	if err != nil {
		return nil, err
	}
	return NewPostgresStore(ctx, dsn)
}

func (s *PostgresStore) Close() error {
	return s.db.Close()
}

func (s *PostgresStore) CreatePet(ctx context.Context, pet Pet) (Pet, error) {
	if err := validatePetForSave(pet); err != nil {
		return Pet{}, err
	}
	if pet.Id == 0 {
		id, err := s.nextID(ctx, "pet_id_seq")
		if err != nil {
			return Pet{}, err
		}
		pet.Id = id
	}
	return s.upsertPet(ctx, pet)
}

func (s *PostgresStore) UpdatePet(ctx context.Context, pet Pet) (Pet, error) {
	if pet.Id == 0 {
		return Pet{}, ErrInvalidInput
	}
	if err := validatePetForSave(pet); err != nil {
		return Pet{}, err
	}
	if _, err := s.GetPetByID(ctx, pet.Id); err != nil {
		return Pet{}, err
	}
	return s.upsertPet(ctx, pet)
}

func (s *PostgresStore) GetPetByID(ctx context.Context, id int64) (Pet, error) {
	row := s.db.QueryRowContext(ctx, `
		SELECT id, name, category, photo_urls, tags, status::text
		FROM pet
		WHERE id = $1`, id)
	return scanPet(row)
}

func (s *PostgresStore) DeletePet(ctx context.Context, id int64) (bool, error) {
	result, err := s.db.ExecContext(ctx, `DELETE FROM pet WHERE id = $1`, id)
	return rowsAffected(result, err)
}

func (s *PostgresStore) FindPetsByStatus(ctx context.Context, statuses []string) ([]Pet, error) {
	statuses = compactStrings(statuses)
	if len(statuses) == 0 {
		statuses = []string{"available"}
	}
	query, args := inClauseQuery(`
		SELECT id, name, category, photo_urls, tags, status::text
		FROM pet
		WHERE status::text IN (`, statuses, ") ORDER BY id")
	return s.queryPets(ctx, query, args...)
}

func (s *PostgresStore) FindPetsByTags(ctx context.Context, tags []string) ([]Pet, error) {
	tags = compactStrings(tags)
	if len(tags) == 0 {
		return []Pet{}, nil
	}
	query, args := inClauseQuery(`
		SELECT id, name, category, photo_urls, tags, status::text
		FROM pet
		WHERE EXISTS (
			SELECT 1
			FROM json_array_elements(COALESCE(tags, '[]'::json)) elem
			WHERE elem->>'name' IN (`, tags, ")) ORDER BY id")
	return s.queryPets(ctx, query, args...)
}

func (s *PostgresStore) UpdatePetFields(ctx context.Context, id int64, name *string, status *string) (bool, error) {
	if name == nil && status == nil {
		_, err := s.GetPetByID(ctx, id)
		return err == nil, err
	}

	sets := []string{}
	args := []any{}
	if name != nil {
		args = append(args, *name)
		sets = append(sets, fmt.Sprintf("name = $%d", len(args)))
	}
	if status != nil {
		if !validPetStatus(*status) {
			return false, ErrInvalidInput
		}
		args = append(args, *status)
		sets = append(sets, fmt.Sprintf("status = NULLIF($%d, '')::pet_status", len(args)))
	}
	args = append(args, id)

	result, err := s.db.ExecContext(ctx, "UPDATE pet SET "+strings.Join(sets, ", ")+" WHERE id = $"+strconv.Itoa(len(args)), args...)
	return rowsAffected(result, err)
}

func (s *PostgresStore) SavePetPhoto(ctx context.Context, petID int64, content []byte, contentType string, metadata string) error {
	var metadataArg any
	if metadata != "" {
		metadataArg = metadata
	}
	_, err := s.db.ExecContext(ctx, `
		INSERT INTO pet_photo (id, pet_id, content_type, metadata, content)
		VALUES (nextval('pet_photo_id_seq'), $1, $2, $3, $4)`,
		petID, contentType, metadataArg, content)
	return err
}

func (s *PostgresStore) Inventory(ctx context.Context) (map[string]int32, error) {
	rows, err := s.db.QueryContext(ctx, `
		SELECT status::text, COUNT(*)
		FROM pet
		WHERE status IS NOT NULL
		GROUP BY status
		ORDER BY status`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	inventory := map[string]int32{}
	for rows.Next() {
		var status string
		var count int32
		if err := rows.Scan(&status, &count); err != nil {
			return nil, err
		}
		inventory[status] = count
	}
	return inventory, rows.Err()
}

func (s *PostgresStore) CreateOrder(ctx context.Context, order Order) (Order, error) {
	if order.Id == 0 {
		id, err := s.nextID(ctx, "order_id_seq")
		if err != nil {
			return Order{}, err
		}
		order.Id = id
	}
	return s.upsertOrder(ctx, order)
}

func (s *PostgresStore) GetOrderByID(ctx context.Context, id int64) (Order, error) {
	row := s.db.QueryRowContext(ctx, `
		SELECT id, pet_id, quantity, ship_date, status::text, complete
		FROM "order"
		WHERE id = $1`, id)
	return scanOrder(row)
}

func (s *PostgresStore) DeleteOrder(ctx context.Context, id int64) (bool, error) {
	result, err := s.db.ExecContext(ctx, `DELETE FROM "order" WHERE id = $1`, id)
	return rowsAffected(result, err)
}

func (s *PostgresStore) CreateUser(ctx context.Context, user User) (User, error) {
	if user.Username == "" {
		return User{}, ErrInvalidInput
	}
	if user.Id == 0 {
		id, err := s.nextID(ctx, "user_id_seq")
		if err != nil {
			return User{}, err
		}
		user.Id = id
	}
	return s.upsertUser(ctx, user)
}

func (s *PostgresStore) CreateUsers(ctx context.Context, users []User) (User, error) {
	var last User
	tx, err := s.db.BeginTx(ctx, nil)
	if err != nil {
		return User{}, err
	}
	defer tx.Rollback()

	for _, user := range users {
		if user.Username == "" {
			return User{}, ErrInvalidInput
		}
		if user.Id == 0 {
			id, err := nextIDTx(ctx, tx, "user_id_seq")
			if err != nil {
				return User{}, err
			}
			user.Id = id
		}
		row, err := execUpsertUser(ctx, tx, user)
		if err != nil {
			return User{}, err
		}
		last, err = scanUser(row)
		if err != nil {
			return User{}, err
		}
	}
	if err := tx.Commit(); err != nil {
		return User{}, err
	}
	return last, nil
}

func (s *PostgresStore) GetUserByUsername(ctx context.Context, username string) (User, error) {
	row := s.db.QueryRowContext(ctx, `
		SELECT id, username, first_name, last_name, email, password, phone, user_status
		FROM "user"
		WHERE username = $1`, username)
	return scanUser(row)
}

func (s *PostgresStore) UpdateUser(ctx context.Context, username string, user User) (bool, error) {
	if username == "" {
		return false, ErrInvalidInput
	}
	if _, err := s.GetUserByUsername(ctx, username); err != nil {
		return false, err
	}
	if user.Username == "" {
		user.Username = username
	}
	if user.Id == 0 {
		existing, err := s.GetUserByUsername(ctx, username)
		if err != nil {
			return false, err
		}
		user.Id = existing.Id
	}

	_, err := s.db.ExecContext(ctx, `
		UPDATE "user"
		SET id = $1, username = $2, first_name = $3, last_name = $4, email = $5, password = $6, phone = $7, user_status = $8
		WHERE username = $9`,
		user.Id, user.Username, user.FirstName, user.LastName, user.Email, user.Password, user.Phone, user.UserStatus, username)
	if err != nil {
		return false, err
	}
	return true, nil
}

func (s *PostgresStore) DeleteUser(ctx context.Context, username string) (bool, error) {
	result, err := s.db.ExecContext(ctx, `DELETE FROM "user" WHERE username = $1`, username)
	return rowsAffected(result, err)
}

func (s *PostgresStore) AuthenticateUser(ctx context.Context, username string, password string) (bool, error) {
	var exists bool
	err := s.db.QueryRowContext(ctx, `
		SELECT EXISTS (
			SELECT 1
			FROM "user"
			WHERE username = $1 AND password = $2
		)`, username, password).Scan(&exists)
	return exists, err
}

func (s *PostgresStore) upsertPet(ctx context.Context, pet Pet) (Pet, error) {
	category, err := json.Marshal(pet.Category)
	if err != nil {
		return Pet{}, err
	}
	photoURLs, err := json.Marshal(pet.PhotoUrls)
	if err != nil {
		return Pet{}, err
	}
	tags, err := json.Marshal(pet.Tags)
	if err != nil {
		return Pet{}, err
	}

	row := s.db.QueryRowContext(ctx, `
		INSERT INTO pet (id, name, category, photo_urls, tags, status)
		VALUES ($1, $2, $3, $4, $5, NULLIF($6, '')::pet_status)
		ON CONFLICT (id) DO UPDATE
		SET name = EXCLUDED.name,
		    category = EXCLUDED.category,
		    photo_urls = EXCLUDED.photo_urls,
		    tags = EXCLUDED.tags,
		    status = EXCLUDED.status
		RETURNING id, name, category, photo_urls, tags, status::text`,
		pet.Id, pet.Name, string(category), string(photoURLs), string(tags), pet.Status)
	return scanPet(row)
}

func (s *PostgresStore) upsertOrder(ctx context.Context, order Order) (Order, error) {
	var shipDate any
	if !order.ShipDate.IsZero() {
		shipDate = order.ShipDate
	}
	row := s.db.QueryRowContext(ctx, `
		INSERT INTO "order" (id, pet_id, quantity, ship_date, status, complete)
		VALUES ($1, $2, $3, $4, NULLIF($5, '')::order_status, $6)
		ON CONFLICT (id) DO UPDATE
		SET pet_id = EXCLUDED.pet_id,
		    quantity = EXCLUDED.quantity,
		    ship_date = EXCLUDED.ship_date,
		    status = EXCLUDED.status,
		    complete = EXCLUDED.complete
		RETURNING id, pet_id, quantity, ship_date, status::text, complete`,
		order.Id, order.PetId, order.Quantity, shipDate, order.Status, order.Complete)
	return scanOrder(row)
}

func (s *PostgresStore) upsertUser(ctx context.Context, user User) (User, error) {
	row, err := execUpsertUser(ctx, s.db, user)
	if err != nil {
		return User{}, err
	}
	return scanUser(row)
}

func execUpsertUser(ctx context.Context, queryer interface {
	QueryRowContext(context.Context, string, ...any) *sql.Row
}, user User) (*sql.Row, error) {
	return queryer.QueryRowContext(ctx, `
		INSERT INTO "user" (id, username, first_name, last_name, email, password, phone, user_status)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
		ON CONFLICT (username) DO UPDATE
		SET id = EXCLUDED.id,
		    first_name = EXCLUDED.first_name,
		    last_name = EXCLUDED.last_name,
		    email = EXCLUDED.email,
		    password = EXCLUDED.password,
		    phone = EXCLUDED.phone,
		    user_status = EXCLUDED.user_status
		RETURNING id, username, first_name, last_name, email, password, phone, user_status`,
		user.Id, user.Username, user.FirstName, user.LastName, user.Email, user.Password, user.Phone, user.UserStatus), nil
}

func (s *PostgresStore) queryPets(ctx context.Context, query string, args ...any) ([]Pet, error) {
	rows, err := s.db.QueryContext(ctx, query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var pets []Pet
	for rows.Next() {
		pet, err := scanPet(rows)
		if err != nil {
			return nil, err
		}
		pets = append(pets, pet)
	}
	return pets, rows.Err()
}

func (s *PostgresStore) nextID(ctx context.Context, seq string) (int64, error) {
	var id int64
	err := s.db.QueryRowContext(ctx, fmt.Sprintf(`SELECT nextval('%s')`, seq)).Scan(&id)
	return id, err
}

func nextIDTx(ctx context.Context, tx *sql.Tx, seq string) (int64, error) {
	var id int64
	err := tx.QueryRowContext(ctx, fmt.Sprintf(`SELECT nextval('%s')`, seq)).Scan(&id)
	return id, err
}

type scanner interface {
	Scan(dest ...any) error
}

func scanPet(row scanner) (Pet, error) {
	var pet Pet
	var category sql.NullString
	var photoURLs []byte
	var tags []byte
	var status sql.NullString

	err := row.Scan(&pet.Id, &pet.Name, &category, &photoURLs, &tags, &status)
	if err != nil {
		return Pet{}, storeError(err)
	}
	if category.Valid && category.String != "" {
		if err := json.Unmarshal([]byte(category.String), &pet.Category); err != nil {
			return Pet{}, err
		}
	}
	if len(photoURLs) > 0 {
		if err := json.Unmarshal(photoURLs, &pet.PhotoUrls); err != nil {
			return Pet{}, err
		}
	}
	if len(tags) > 0 {
		if err := json.Unmarshal(tags, &pet.Tags); err != nil {
			return Pet{}, err
		}
	}
	if status.Valid {
		pet.Status = status.String
	}
	return pet, nil
}

func scanOrder(row scanner) (Order, error) {
	var order Order
	var shipDate sql.NullTime
	var status sql.NullString
	var complete sql.NullBool
	err := row.Scan(&order.Id, &order.PetId, &order.Quantity, &shipDate, &status, &complete)
	if err != nil {
		return Order{}, storeError(err)
	}
	if shipDate.Valid {
		order.ShipDate = shipDate.Time
	}
	if status.Valid {
		order.Status = status.String
	}
	if complete.Valid {
		order.Complete = complete.Bool
	}
	return order, nil
}

func scanUser(row scanner) (User, error) {
	var user User
	var id sql.NullInt64
	var username, firstName, lastName, email, password, phone sql.NullString
	var userStatus sql.NullInt64
	err := row.Scan(&id, &username, &firstName, &lastName, &email, &password, &phone, &userStatus)
	if err != nil {
		return User{}, storeError(err)
	}
	if id.Valid {
		user.Id = id.Int64
	}
	if username.Valid {
		user.Username = username.String
	}
	if firstName.Valid {
		user.FirstName = firstName.String
	}
	if lastName.Valid {
		user.LastName = lastName.String
	}
	if email.Valid {
		user.Email = email.String
	}
	if password.Valid {
		user.Password = password.String
	}
	if phone.Valid {
		user.Phone = phone.String
	}
	if userStatus.Valid {
		user.UserStatus = int32(userStatus.Int64)
	}
	return user, nil
}

func storeError(err error) error {
	if errors.Is(err, sql.ErrNoRows) {
		return ErrNotFound
	}
	return err
}

func rowsAffected(result sql.Result, err error) (bool, error) {
	if err != nil {
		return false, err
	}
	count, err := result.RowsAffected()
	if err != nil {
		return false, err
	}
	return count > 0, nil
}

func validatePetForSave(pet Pet) error {
	if pet.Name == "" || len(pet.PhotoUrls) == 0 {
		return ErrInvalidInput
	}
	if pet.Status != "" && !validPetStatus(pet.Status) {
		return ErrInvalidInput
	}
	return nil
}

func validPetStatus(status string) bool {
	switch status {
	case "available", "pending", "sold":
		return true
	default:
		return false
	}
}

func validOrderStatus(status string) bool {
	switch status {
	case "", "placed", "approved", "delivered":
		return true
	default:
		return false
	}
}

func compactStrings(values []string) []string {
	var compacted []string
	for _, value := range values {
		for _, part := range strings.Split(value, ",") {
			part = strings.TrimSpace(part)
			if part != "" {
				compacted = append(compacted, part)
			}
		}
	}
	return compacted
}

func inClauseQuery(prefix string, values []string, suffix string) (string, []any) {
	args := make([]any, len(values))
	placeholders := make([]string, len(values))
	for i, value := range values {
		args[i] = value
		placeholders[i] = "$" + strconv.Itoa(i+1)
	}
	return prefix + strings.Join(placeholders, ", ") + suffix, args
}

func postgresDSNFromEnv() (string, error) {
	if dsn := os.Getenv("DATABASE_URL"); dsn != "" {
		return dsn, nil
	}
	if dsn := os.Getenv("POSTGRES_DSN"); dsn != "" {
		return dsn, nil
	}

	password := os.Getenv("POSTGRES_PASSWORD")
	if password == "" {
		return "", fmt.Errorf("POSTGRES_PASSWORD is required unless DATABASE_URL or POSTGRES_DSN is set")
	}

	user := envDefault("POSTGRES_USER", "postgres")
	host := envDefault("POSTGRES_HOST", "localhost")
	port := envDefault("POSTGRES_PORT", "5432")
	database := envDefault("POSTGRES_DB", "go-gin-server")

	values := url.Values{}
	values.Set("sslmode", envDefault("POSTGRES_SSLMODE", "disable"))
	return fmt.Sprintf("postgres://%s:%s@%s:%s/%s?%s",
		url.QueryEscape(user),
		url.QueryEscape(password),
		host,
		port,
		url.PathEscape(database),
		values.Encode()), nil
}

func envDefault(name string, fallback string) string {
	value := os.Getenv(name)
	if value == "" {
		return fallback
	}
	return value
}

var _ Store = (*PostgresStore)(nil)
