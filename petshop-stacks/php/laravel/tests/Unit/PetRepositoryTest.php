<?php

declare(strict_types=1);

namespace Tests\Unit;

use App\Repositories\InMemoryPetstoreRepository;
use PHPUnit\Framework\TestCase;

class PetRepositoryTest extends TestCase
{
    private InMemoryPetstoreRepository $repo;

    protected function setUp(): void
    {
        $this->repo = new InMemoryPetstoreRepository();
    }

    // ── addPet ────────────────────────────────────────────────────────────

    public function testAddPetAssignsId(): void
    {
        $pet = $this->repo->addPet(['name' => 'Fido', 'photoUrls' => [], 'status' => 'available']);
        self::assertGreaterThan(0, $pet['id']);
    }

    public function testAddPetPreservesProvidedId(): void
    {
        $pet = $this->repo->addPet(['id' => 42, 'name' => 'Fido', 'photoUrls' => []]);
        self::assertSame(42, $pet['id']);
    }

    public function testAddPetStoresAllFields(): void
    {
        $pet = $this->repo->addPet([
            'name'      => 'Rex',
            'status'    => 'pending',
            'photoUrls' => ['http://example.com/photo.jpg'],
            'tags'      => [['id' => 1, 'name' => 'big']],
            'category'  => ['id' => 1, 'name' => 'Dogs'],
        ]);
        self::assertSame('Rex', $pet['name']);
        self::assertSame('pending', $pet['status']);
        self::assertCount(1, $pet['photoUrls']);
        self::assertCount(1, $pet['tags']);
        self::assertSame('Dogs', $pet['category']['name']);
    }

    public function testAddPetAutoIncrementsId(): void
    {
        $p1 = $this->repo->addPet(['name' => 'A', 'photoUrls' => []]);
        $p2 = $this->repo->addPet(['name' => 'B', 'photoUrls' => []]);
        self::assertGreaterThan($p1['id'], $p2['id']);
    }

    public function testAddPetFirstIdIsOne(): void
    {
        $pet = $this->repo->addPet(['name' => 'First', 'photoUrls' => []]);
        self::assertSame(1, $pet['id']);
    }

    public function testAddPetIdsAreConsecutive(): void
    {
        $p1 = $this->repo->addPet(['name' => 'A', 'photoUrls' => []]);
        $p2 = $this->repo->addPet(['name' => 'B', 'photoUrls' => []]);
        self::assertSame($p1['id'] + 1, $p2['id']);
    }

    // ── updatePet ─────────────────────────────────────────────────────────

    public function testUpdatePetUpdatesExisting(): void
    {
        $pet = $this->repo->addPet(['name' => 'Fido', 'photoUrls' => [], 'status' => 'available']);
        $pet['name']   = 'Max';
        $pet['status'] = 'sold';
        $updated = $this->repo->updatePet($pet);
        self::assertSame('Max', $updated['name']);
        self::assertSame('sold', $updated['status']);
    }

    public function testUpdatePetReturnsNullForMissingId(): void
    {
        $result = $this->repo->updatePet(['id' => 9999, 'name' => 'Ghost', 'photoUrls' => []]);
        self::assertNull($result);
    }

    public function testUpdatePetReturnsNullWhenIdMissing(): void
    {
        $result = $this->repo->updatePet(['name' => 'Ghost', 'photoUrls' => []]);
        self::assertNull($result);
    }

    // ── getPetById ────────────────────────────────────────────────────────

    public function testGetPetByIdReturnsCorrectPet(): void
    {
        $pet = $this->repo->addPet(['name' => 'Spot', 'photoUrls' => []]);
        $found = $this->repo->getPetById($pet['id']);
        self::assertSame($pet['id'], $found['id']);
    }

    public function testGetPetByIdReturnsNullForUnknown(): void
    {
        self::assertNull($this->repo->getPetById(99999));
    }

    // ── findPetsByStatus ──────────────────────────────────────────────────

    public function testFindPetsByStatusFiltersCorrectly(): void
    {
        $this->repo->addPet(['name' => 'A', 'photoUrls' => [], 'status' => 'available']);
        $this->repo->addPet(['name' => 'B', 'photoUrls' => [], 'status' => 'pending']);
        $this->repo->addPet(['name' => 'C', 'photoUrls' => [], 'status' => 'available']);
        $result = $this->repo->findPetsByStatus('available');
        self::assertCount(2, $result);
    }

    public function testFindPetsByStatusReturnsEmptyArrayForUnknownStatus(): void
    {
        $this->repo->addPet(['name' => 'A', 'photoUrls' => [], 'status' => 'available']);
        $result = $this->repo->findPetsByStatus('sold');
        self::assertEmpty($result);
    }

    public function testFindPetsByStatusReturnsIndexedArray(): void
    {
        $this->repo->addPet(['name' => 'Ax', 'photoUrls' => [], 'status' => 'available']);
        $this->repo->addPet(['name' => 'Bx', 'photoUrls' => [], 'status' => 'pending']);
        $this->repo->addPet(['name' => 'Cx', 'photoUrls' => [], 'status' => 'available']);
        $result = $this->repo->findPetsByStatus('available');
        self::assertArrayHasKey(0, $result);
        self::assertArrayHasKey(1, $result);
        self::assertSame('Ax', $result[0]['name']);
        self::assertSame('Cx', $result[1]['name']);
    }

    // ── findPetsByTags ────────────────────────────────────────────────────

    public function testFindPetsByTagsFiltersCorrectly(): void
    {
        $this->repo->addPet(['name' => 'A', 'photoUrls' => [], 'tags' => [['id' => 1, 'name' => 'big']]]);
        $this->repo->addPet(['name' => 'B', 'photoUrls' => [], 'tags' => [['id' => 2, 'name' => 'small']]]);
        $result = $this->repo->findPetsByTags(['big']);
        self::assertCount(1, $result);
        self::assertSame('A', $result[0]['name']);
    }

    public function testFindPetsByTagsEmptyTagsReturnsAll(): void
    {
        $this->repo->addPet(['name' => 'A', 'photoUrls' => []]);
        $this->repo->addPet(['name' => 'B', 'photoUrls' => []]);
        $result = $this->repo->findPetsByTags([]);
        self::assertCount(2, $result);
    }

    public function testFindPetsByTagsEmptyTagsReturnsIndexedArray(): void
    {
        $this->repo->addPet(['name' => 'TagA', 'photoUrls' => []]);
        $this->repo->addPet(['name' => 'TagB', 'photoUrls' => []]);
        $result = $this->repo->findPetsByTags([]);
        self::assertArrayHasKey(0, $result);
        self::assertArrayHasKey(1, $result);
        self::assertSame('TagA', $result[0]['name']);
        self::assertSame('TagB', $result[1]['name']);
    }

    // ── updatePetWithForm ─────────────────────────────────────────────────

    public function testUpdatePetWithFormUpdatesBothFields(): void
    {
        $pet = $this->repo->addPet(['name' => 'Fido', 'photoUrls' => [], 'status' => 'available']);
        $ok  = $this->repo->updatePetWithForm($pet['id'], 'Max', 'sold');
        self::assertTrue($ok);
        $updated = $this->repo->getPetById($pet['id']);
        self::assertSame('Max', $updated['name']);
        self::assertSame('sold', $updated['status']);
    }

    public function testUpdatePetWithFormReturnsFalseForUnknown(): void
    {
        self::assertFalse($this->repo->updatePetWithForm(99999, 'X', 'available'));
    }

    public function testUpdatePetWithFormSkipsNullFields(): void
    {
        $pet = $this->repo->addPet(['name' => 'Fido', 'photoUrls' => [], 'status' => 'available']);
        $this->repo->updatePetWithForm($pet['id'], null, null);
        $updated = $this->repo->getPetById($pet['id']);
        self::assertSame('Fido', $updated['name']);
        self::assertSame('available', $updated['status']);
    }

    // ── deletePet ─────────────────────────────────────────────────────────

    public function testDeletePetRemovesPet(): void
    {
        $pet = $this->repo->addPet(['name' => 'Goner', 'photoUrls' => []]);
        self::assertTrue($this->repo->deletePet($pet['id']));
        self::assertNull($this->repo->getPetById($pet['id']));
    }

    public function testDeletePetReturnsFalseForUnknown(): void
    {
        self::assertFalse($this->repo->deletePet(99999));
    }

    // ── uploadFile ────────────────────────────────────────────────────────

    public function testUploadFileReturnsContentLength(): void
    {
        $pet  = $this->repo->addPet(['name' => 'Snap', 'photoUrls' => []]);
        $data = 'binary-image-data';
        $bytes = $this->repo->uploadFile($pet['id'], $data);
        self::assertSame(strlen($data), $bytes);
    }

    // ── inventory (pet status counts) ────────────────────────────────────

    public function testInventoryCountsPetsByStatus(): void
    {
        $this->repo->addPet(['name' => 'A', 'photoUrls' => [], 'status' => 'available']);
        $this->repo->addPet(['name' => 'B', 'photoUrls' => [], 'status' => 'available']);
        $this->repo->addPet(['name' => 'C', 'photoUrls' => [], 'status' => 'sold']);
        $inv = $this->repo->getInventory();
        self::assertSame(2, $inv['available']);
        self::assertSame(1, $inv['sold']);
    }
}
