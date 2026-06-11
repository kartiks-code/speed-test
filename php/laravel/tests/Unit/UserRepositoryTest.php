<?php

declare(strict_types=1);

namespace Tests\Unit;

use App\Repositories\InMemoryPetstoreRepository;
use PHPUnit\Framework\TestCase;

class UserRepositoryTest extends TestCase
{
    private InMemoryPetstoreRepository $repo;

    protected function setUp(): void
    {
        $this->repo = new InMemoryPetstoreRepository();
    }

    // ── createUser ────────────────────────────────────────────────────────

    public function testCreateUserAssignsId(): void
    {
        $user = $this->repo->createUser(['username' => 'alice', 'password' => 'secret']);
        self::assertGreaterThan(0, $user['id']);
    }

    public function testCreateUserPreservesProvidedId(): void
    {
        $user = $this->repo->createUser(['id' => 55, 'username' => 'bob', 'password' => 'pw']);
        self::assertSame(55, $user['id']);
    }

    public function testCreateUserStoresAllFields(): void
    {
        $user = $this->repo->createUser([
            'username'   => 'carol',
            'firstName'  => 'Carol',
            'lastName'   => 'Smith',
            'email'      => 'carol@example.com',
            'password'   => 'pw123',
            'phone'      => '555-1234',
            'userStatus' => 1,
        ]);
        self::assertSame('carol', $user['username']);
        self::assertSame('Carol', $user['firstName']);
        self::assertSame('carol@example.com', $user['email']);
        self::assertSame(1, $user['userStatus']);
    }

    public function testCreateUserAutoIncrementsId(): void
    {
        $u1 = $this->repo->createUser(['username' => 'u1', 'password' => 'pw']);
        $u2 = $this->repo->createUser(['username' => 'u2', 'password' => 'pw']);
        self::assertGreaterThan($u1['id'], $u2['id']);
    }

    // ── getUserByName ─────────────────────────────────────────────────────

    public function testGetUserByNameReturnsCorrectUser(): void
    {
        $this->repo->createUser(['username' => 'dave', 'password' => 'pw']);
        $found = $this->repo->getUserByName('dave');
        self::assertSame('dave', $found['username']);
    }

    public function testGetUserByNameReturnsNullForUnknown(): void
    {
        self::assertNull($this->repo->getUserByName('nobody'));
    }

    // ── updateUser ────────────────────────────────────────────────────────

    public function testUpdateUserModifiesUser(): void
    {
        $this->repo->createUser(['username' => 'eve', 'firstName' => 'Eve', 'password' => 'pw']);
        $ok = $this->repo->updateUser('eve', ['firstName' => 'Evelyn', 'password' => 'newpw']);
        self::assertTrue($ok);
        $updated = $this->repo->getUserByName('eve');
        self::assertSame('Evelyn', $updated['firstName']);
    }

    public function testUpdateUserReturnsFalseForUnknown(): void
    {
        self::assertFalse($this->repo->updateUser('ghost', ['firstName' => 'X', 'password' => 'pw']));
    }

    public function testUpdateUserPreservesUsername(): void
    {
        $this->repo->createUser(['username' => 'frank', 'password' => 'pw']);
        $this->repo->updateUser('frank', ['firstName' => 'Francis', 'password' => 'pw2']);
        $updated = $this->repo->getUserByName('frank');
        self::assertSame('frank', $updated['username']);
    }

    // ── deleteUser ────────────────────────────────────────────────────────

    public function testDeleteUserRemovesUser(): void
    {
        $this->repo->createUser(['username' => 'gina', 'password' => 'pw']);
        self::assertTrue($this->repo->deleteUser('gina'));
        self::assertNull($this->repo->getUserByName('gina'));
    }

    public function testDeleteUserReturnsFalseForUnknown(): void
    {
        self::assertFalse($this->repo->deleteUser('nobody'));
    }

    // ── loginUser ─────────────────────────────────────────────────────────

    public function testLoginUserReturnsTokenForValidCredentials(): void
    {
        $this->repo->createUser(['username' => 'hal', 'password' => 'hal123']);
        $token = $this->repo->loginUser('hal', 'hal123');
        self::assertNotNull($token);
        self::assertStringContainsString('hal', $token);
    }

    public function testLoginUserReturnsNullForWrongPassword(): void
    {
        $this->repo->createUser(['username' => 'iris', 'password' => 'correct']);
        self::assertNull($this->repo->loginUser('iris', 'wrong'));
    }

    public function testLoginUserReturnsNullForUnknownUser(): void
    {
        self::assertNull($this->repo->loginUser('unknown', 'pw'));
    }

    // ── createUsersWithListInput (multiple) ───────────────────────────────

    public function testCreateMultipleUsersPreservesAll(): void
    {
        $users = [
            ['username' => 'user1', 'password' => 'pw1'],
            ['username' => 'user2', 'password' => 'pw2'],
            ['username' => 'user3', 'password' => 'pw3'],
        ];
        foreach ($users as $u) {
            $this->repo->createUser($u);
        }
        self::assertNotNull($this->repo->getUserByName('user1'));
        self::assertNotNull($this->repo->getUserByName('user2'));
        self::assertNotNull($this->repo->getUserByName('user3'));
    }
}
