<?php

declare(strict_types=1);

namespace App\Repositories;

class InMemoryPetstoreRepository implements PetstoreRepositoryInterface
{
    /** @var array<int,array> */
    private array $pets = [];

    /** @var array<int,array> */
    private array $orders = [];

    /** @var array<string,array> */
    private array $users = [];

    /** @var array<int,string> pet_id => binary content */
    private array $photos = [];

    // ── Pet ──────────────────────────────────────────────────────────────

    public function addPet(array $pet): array
    {
        if (empty($pet['id'])) {
            $pet['id'] = $this->nextPetId();
        }
        $this->pets[$pet['id']] = $pet;
        return $pet;
    }

    public function updatePet(array $pet): ?array
    {
        if (empty($pet['id']) || !isset($this->pets[$pet['id']])) {
            return null;
        }
        $this->pets[$pet['id']] = $pet;
        return $pet;
    }

    public function findPetsByStatus(string $status): array
    {
        return array_values(array_filter(
            $this->pets,
            fn(array $p) => ($p['status'] ?? '') === $status
        ));
    }

    public function findPetsByTags(array $tags): array
    {
        if (empty($tags)) {
            return array_values($this->pets);
        }
        return array_values(array_filter($this->pets, function (array $p) use ($tags) {
            $petTags = array_column($p['tags'] ?? [], 'name');
            return (bool) array_intersect($tags, $petTags);
        }));
    }

    public function getPetById(int $petId): ?array
    {
        return $this->pets[$petId] ?? null;
    }

    public function updatePetWithForm(int $petId, ?string $name, ?string $status): bool
    {
        if (!isset($this->pets[$petId])) {
            return false;
        }
        if ($name !== null && $name !== '') {
            $this->pets[$petId]['name'] = $name;
        }
        if ($status !== null && $status !== '') {
            $this->pets[$petId]['status'] = $status;
        }
        return true;
    }

    public function deletePet(int $petId): bool
    {
        if (!isset($this->pets[$petId])) {
            return false;
        }
        unset($this->pets[$petId]);
        return true;
    }

    public function uploadFile(int $petId, string $content): int
    {
        $this->photos[$petId] = $content;
        return strlen($content);
    }

    // ── Store ─────────────────────────────────────────────────────────────

    public function getInventory(): array
    {
        $counts = [];
        foreach ($this->pets as $pet) {
            $status = $pet['status'] ?? 'available';
            $counts[$status] = ($counts[$status] ?? 0) + 1;
        }
        return $counts;
    }

    public function placeOrder(array $order): array
    {
        if (empty($order['id'])) {
            $order['id'] = $this->nextOrderId();
        }
        $this->orders[$order['id']] = $order;
        return $order;
    }

    public function getOrderById(int $orderId): ?array
    {
        return $this->orders[$orderId] ?? null;
    }

    public function deleteOrder(int $orderId): bool
    {
        if (!isset($this->orders[$orderId])) {
            return false;
        }
        unset($this->orders[$orderId]);
        return true;
    }

    // ── User ──────────────────────────────────────────────────────────────

    public function createUser(array $user): array
    {
        if (empty($user['id'])) {
            $user['id'] = $this->nextUserId();
        }
        $this->users[$user['username']] = $user;
        return $user;
    }

    public function getUserByName(string $username): ?array
    {
        return $this->users[$username] ?? null;
    }

    public function updateUser(string $username, array $user): bool
    {
        if (!isset($this->users[$username])) {
            return false;
        }
        $user['username'] = $username;
        if (empty($user['id'])) {
            $user['id'] = $this->users[$username]['id'];
        }
        $this->users[$username] = $user;
        return true;
    }

    public function deleteUser(string $username): bool
    {
        if (!isset($this->users[$username])) {
            return false;
        }
        unset($this->users[$username]);
        return true;
    }

    public function loginUser(string $username, string $password): ?string
    {
        $user = $this->users[$username] ?? null;
        if ($user === null || ($user['password'] ?? '') !== $password) {
            return null;
        }
        return 'logged-in-token-' . $username;
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    private function nextPetId(): int
    {
        return empty($this->pets) ? 1 : max(array_keys($this->pets)) + 1;
    }

    private function nextOrderId(): int
    {
        return empty($this->orders) ? 1 : max(array_keys($this->orders)) + 1;
    }

    private function nextUserId(): int
    {
        return empty($this->users) ? 1 : max(array_column($this->users, 'id')) + 1;
    }
}
