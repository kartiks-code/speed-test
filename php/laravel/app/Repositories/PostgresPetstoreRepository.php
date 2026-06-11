<?php

declare(strict_types=1);

namespace App\Repositories;

use PDO;
use PDOException;

class PostgresPetstoreRepository implements PetstoreRepositoryInterface
{
    private PDO $pdo;

    public function __construct()
    {
        $host     = env('DB_HOST', 'localhost');
        $port     = env('DB_PORT', '5434');
        $database = env('DB_DATABASE', 'php-laravel');
        $username = env('DB_USERNAME', 'myuser');
        $password = env('DB_PASSWORD', 'mypassword');

        $dsn = "pgsql:host={$host};port={$port};dbname={$database}";
        $this->pdo = new PDO($dsn, $username, $password, [
            PDO::ATTR_ERRMODE            => PDO::ERRMODE_EXCEPTION,
            PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
        ]);
    }

    // ── Pet ──────────────────────────────────────────────────────────────

    public function addPet(array $pet): array
    {
        $id        = $pet['id'] ?? null;
        $name      = $pet['name'] ?? '';
        $status    = $pet['status'] ?? 'available';
        $category  = isset($pet['category']) ? json_encode($pet['category']) : null;
        $photoUrls = json_encode($pet['photoUrls'] ?? []);
        $tags      = json_encode($pet['tags'] ?? []);

        if ($id === null || $id === 0) {
            $stmt = $this->pdo->query('SELECT COALESCE(MAX(id),0)+1 AS next_id FROM pet');
            $id   = (int) $stmt->fetchColumn();
        }

        $sql = <<<SQL
            INSERT INTO pet (id, name, status, category, photo_urls, tags)
            VALUES (:id, :name, :status::pet_status, :category, :photo_urls, :tags)
            ON CONFLICT (id) DO UPDATE
                SET name       = EXCLUDED.name,
                    status     = EXCLUDED.status,
                    category   = EXCLUDED.category,
                    photo_urls = EXCLUDED.photo_urls,
                    tags       = EXCLUDED.tags
            RETURNING id, name, status::text, category, photo_urls, tags
        SQL;

        $stmt = $this->pdo->prepare($sql);
        $stmt->execute([
            ':id'         => $id,
            ':name'       => $name,
            ':status'     => $status,
            ':category'   => $category,
            ':photo_urls' => $photoUrls,
            ':tags'       => $tags,
        ]);

        return $this->hydratePet($stmt->fetch());
    }

    public function updatePet(array $pet): ?array
    {
        if (empty($pet['id'])) {
            return null;
        }
        return $this->addPet($pet);
    }

    public function findPetsByStatus(string $status): array
    {
        $stmt = $this->pdo->prepare(
            'SELECT id, name, status::text, category, photo_urls, tags FROM pet WHERE status = :status::pet_status'
        );
        $stmt->execute([':status' => $status]);
        return array_map([$this, 'hydratePet'], $stmt->fetchAll());
    }

    public function findPetsByTags(array $tags): array
    {
        if (empty($tags)) {
            $stmt = $this->pdo->query('SELECT id, name, status::text, category, photo_urls, tags FROM pet');
            return array_map([$this, 'hydratePet'], $stmt->fetchAll());
        }

        $placeholders = implode(',', array_fill(0, count($tags), '?'));
        $sql = <<<SQL
            SELECT id, name, status::text, category, photo_urls, tags
            FROM pet, jsonb_array_elements(tags::jsonb) AS t
            WHERE t->>'name' IN ({$placeholders})
            GROUP BY id
        SQL;
        $stmt = $this->pdo->prepare($sql);
        $stmt->execute($tags);
        return array_map([$this, 'hydratePet'], $stmt->fetchAll());
    }

    public function getPetById(int $petId): ?array
    {
        $stmt = $this->pdo->prepare(
            'SELECT id, name, status::text, category, photo_urls, tags FROM pet WHERE id = :id'
        );
        $stmt->execute([':id' => $petId]);
        $row = $stmt->fetch();
        return $row ? $this->hydratePet($row) : null;
    }

    public function updatePetWithForm(int $petId, ?string $name, ?string $status): bool
    {
        $existing = $this->getPetById($petId);
        if ($existing === null) {
            return false;
        }
        if ($name !== null && $name !== '') {
            $existing['name'] = $name;
        }
        if ($status !== null && $status !== '') {
            $existing['status'] = $status;
        }
        $this->addPet($existing);
        return true;
    }

    public function deletePet(int $petId): bool
    {
        $stmt = $this->pdo->prepare('DELETE FROM pet WHERE id = :id');
        $stmt->execute([':id' => $petId]);
        return $stmt->rowCount() > 0;
    }

    public function uploadFile(int $petId, string $content): int
    {
        $sql = <<<SQL
            INSERT INTO pet_photo (pet_id, content)
            VALUES (:pet_id, :content)
            ON CONFLICT (pet_id) DO UPDATE SET content = EXCLUDED.content
        SQL;
        $stmt = $this->pdo->prepare($sql);
        $stmt->bindValue(':pet_id', $petId, PDO::PARAM_INT);
        $stmt->bindValue(':content', $content, PDO::PARAM_LOB);
        $stmt->execute();
        return strlen($content);
    }

    // ── Store ─────────────────────────────────────────────────────────────

    public function getInventory(): array
    {
        $stmt = $this->pdo->query(
            'SELECT status::text AS status, COUNT(*) AS cnt FROM pet GROUP BY status'
        );
        $result = [];
        foreach ($stmt->fetchAll() as $row) {
            $result[$row['status']] = (int) $row['cnt'];
        }
        return $result;
    }

    public function placeOrder(array $order): array
    {
        $id       = $order['id'] ?? null;
        $petId    = $order['petId'] ?? 0;
        $quantity = $order['quantity'] ?? 0;
        $shipDate = $order['shipDate'] ?? null;
        $status   = $order['status'] ?? 'placed';
        $complete = $order['complete'] ?? false;

        if ($id === null || $id === 0) {
            $stmt = $this->pdo->query('SELECT COALESCE(MAX(id),0)+1 AS next_id FROM "order"');
            $id   = (int) $stmt->fetchColumn();
        }

        $sql = <<<SQL
            INSERT INTO "order" (id, pet_id, quantity, ship_date, status, complete)
            VALUES (:id, :pet_id, :quantity, :ship_date, :status::order_status, :complete)
            ON CONFLICT (id) DO UPDATE
                SET pet_id    = EXCLUDED.pet_id,
                    quantity  = EXCLUDED.quantity,
                    ship_date = EXCLUDED.ship_date,
                    status    = EXCLUDED.status,
                    complete  = EXCLUDED.complete
            RETURNING id, pet_id, quantity, ship_date, status::text, complete
        SQL;

        $stmt = $this->pdo->prepare($sql);
        $stmt->execute([
            ':id'        => $id,
            ':pet_id'    => $petId,
            ':quantity'  => $quantity,
            ':ship_date' => $shipDate,
            ':status'    => $status,
            ':complete'  => $complete ? 'true' : 'false',
        ]);

        return $this->hydrateOrder($stmt->fetch());
    }

    public function getOrderById(int $orderId): ?array
    {
        $stmt = $this->pdo->prepare(
            'SELECT id, pet_id, quantity, ship_date, status::text, complete FROM "order" WHERE id = :id'
        );
        $stmt->execute([':id' => $orderId]);
        $row = $stmt->fetch();
        return $row ? $this->hydrateOrder($row) : null;
    }

    public function deleteOrder(int $orderId): bool
    {
        $stmt = $this->pdo->prepare('DELETE FROM "order" WHERE id = :id');
        $stmt->execute([':id' => $orderId]);
        return $stmt->rowCount() > 0;
    }

    // ── User ──────────────────────────────────────────────────────────────

    public function createUser(array $user): array
    {
        $id         = $user['id'] ?? null;
        $username   = $user['username'] ?? '';
        $firstName  = $user['firstName'] ?? null;
        $lastName   = $user['lastName'] ?? null;
        $email      = $user['email'] ?? null;
        $password   = $user['password'] ?? null;
        $phone      = $user['phone'] ?? null;
        $userStatus = $user['userStatus'] ?? 0;

        if ($id === null || $id === 0) {
            $stmt = $this->pdo->query('SELECT COALESCE(MAX(id),0)+1 AS next_id FROM "user"');
            $id   = (int) $stmt->fetchColumn();
        }

        $sql = <<<SQL
            INSERT INTO "user" (id, username, first_name, last_name, email, password, phone, user_status)
            VALUES (:id, :username, :first_name, :last_name, :email, :password, :phone, :user_status)
            ON CONFLICT (id) DO UPDATE
                SET username    = EXCLUDED.username,
                    first_name  = EXCLUDED.first_name,
                    last_name   = EXCLUDED.last_name,
                    email       = EXCLUDED.email,
                    password    = EXCLUDED.password,
                    phone       = EXCLUDED.phone,
                    user_status = EXCLUDED.user_status
            RETURNING id, username, first_name, last_name, email, password, phone, user_status
        SQL;

        $stmt = $this->pdo->prepare($sql);
        $stmt->execute([
            ':id'          => $id,
            ':username'    => $username,
            ':first_name'  => $firstName,
            ':last_name'   => $lastName,
            ':email'       => $email,
            ':password'    => $password,
            ':phone'       => $phone,
            ':user_status' => $userStatus,
        ]);

        return $this->hydrateUser($stmt->fetch());
    }

    public function getUserByName(string $username): ?array
    {
        $stmt = $this->pdo->prepare(
            'SELECT id, username, first_name, last_name, email, password, phone, user_status FROM "user" WHERE username = :username'
        );
        $stmt->execute([':username' => $username]);
        $row = $stmt->fetch();
        return $row ? $this->hydrateUser($row) : null;
    }

    public function updateUser(string $username, array $user): bool
    {
        $existing = $this->getUserByName($username);
        if ($existing === null) {
            return false;
        }
        $user['id']       = $existing['id'];
        $user['username'] = $username;
        $this->createUser($user);
        return true;
    }

    public function deleteUser(string $username): bool
    {
        $stmt = $this->pdo->prepare('DELETE FROM "user" WHERE username = :username');
        $stmt->execute([':username' => $username]);
        return $stmt->rowCount() > 0;
    }

    public function loginUser(string $username, string $password): ?string
    {
        $user = $this->getUserByName($username);
        if ($user === null || ($user['password'] ?? '') !== $password) {
            return null;
        }
        return 'token-' . $username . '-' . time();
    }

    // ── Hydration helpers ─────────────────────────────────────────────────

    private function hydratePet(array $row): array
    {
        return [
            'id'        => (int) $row['id'],
            'name'      => $row['name'],
            'status'    => $row['status'],
            'category'  => isset($row['category']) ? json_decode($row['category'], true) : null,
            'photoUrls' => json_decode($row['photo_urls'] ?? '[]', true),
            'tags'      => json_decode($row['tags'] ?? '[]', true),
        ];
    }

    private function hydrateOrder(array $row): array
    {
        return [
            'id'       => (int) $row['id'],
            'petId'    => (int) $row['pet_id'],
            'quantity' => (int) $row['quantity'],
            'shipDate' => $row['ship_date'],
            'status'   => $row['status'],
            'complete' => (bool) $row['complete'],
        ];
    }

    private function hydrateUser(array $row): array
    {
        return [
            'id'         => (int) $row['id'],
            'username'   => $row['username'],
            'firstName'  => $row['first_name'],
            'lastName'   => $row['last_name'],
            'email'      => $row['email'],
            'password'   => $row['password'],
            'phone'      => $row['phone'],
            'userStatus' => (int) $row['user_status'],
        ];
    }
}
