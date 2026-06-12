<?php

declare(strict_types=1);

namespace App\Repositories;

interface PetstoreRepositoryInterface
{
    // ── Pet ──────────────────────────────────────────────────────────────

    public function addPet(array $pet): array;

    public function updatePet(array $pet): ?array;

    public function findPetsByStatus(string $status): array;

    public function findPetsByTags(array $tags): array;

    public function getPetById(int $petId): ?array;

    public function updatePetWithForm(int $petId, ?string $name, ?string $status): bool;

    public function deletePet(int $petId): bool;

    public function uploadFile(int $petId, string $content): int;

    // ── Store ─────────────────────────────────────────────────────────────

    public function getInventory(): array;

    public function placeOrder(array $order): array;

    public function getOrderById(int $orderId): ?array;

    public function deleteOrder(int $orderId): bool;

    // ── User ──────────────────────────────────────────────────────────────

    public function createUser(array $user): array;

    public function getUserByName(string $username): ?array;

    public function updateUser(string $username, array $user): bool;

    public function deleteUser(string $username): bool;

    public function loginUser(string $username, string $password): ?string;
}
