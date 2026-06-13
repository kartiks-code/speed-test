<?php

declare(strict_types=1);

namespace Tests\Unit;

use App\Repositories\InMemoryPetstoreRepository;
use PHPUnit\Framework\TestCase;

class StoreRepositoryTest extends TestCase
{
    private InMemoryPetstoreRepository $repo;

    protected function setUp(): void
    {
        $this->repo = new InMemoryPetstoreRepository();
    }

    // ── placeOrder ────────────────────────────────────────────────────────

    public function testPlaceOrderAssignsId(): void
    {
        $order = $this->repo->placeOrder(['petId' => 1, 'quantity' => 2, 'status' => 'placed']);
        self::assertGreaterThan(0, $order['id']);
    }

    public function testPlaceOrderPreservesProvidedId(): void
    {
        $order = $this->repo->placeOrder(['id' => 7, 'petId' => 1, 'quantity' => 1, 'status' => 'placed']);
        self::assertSame(7, $order['id']);
    }

    public function testPlaceOrderStoresAllFields(): void
    {
        $order = $this->repo->placeOrder([
            'petId'    => 10,
            'quantity' => 3,
            'status'   => 'approved',
            'complete' => true,
        ]);
        self::assertSame(10, $order['petId']);
        self::assertSame(3, $order['quantity']);
        self::assertSame('approved', $order['status']);
        self::assertTrue($order['complete']);
    }

    public function testPlaceOrderAutoIncrementsId(): void
    {
        $o1 = $this->repo->placeOrder(['petId' => 1, 'quantity' => 1, 'status' => 'placed']);
        $o2 = $this->repo->placeOrder(['petId' => 2, 'quantity' => 1, 'status' => 'placed']);
        self::assertGreaterThan($o1['id'], $o2['id']);
    }

    public function testFirstOrderIdIsOne(): void
    {
        $order = $this->repo->placeOrder(['petId' => 1, 'quantity' => 1, 'status' => 'placed']);
        self::assertSame(1, $order['id']);
    }

    public function testOrderIdsAreConsecutive(): void
    {
        $o1 = $this->repo->placeOrder(['petId' => 1, 'quantity' => 1, 'status' => 'placed']);
        $o2 = $this->repo->placeOrder(['petId' => 2, 'quantity' => 1, 'status' => 'placed']);
        self::assertSame($o1['id'] + 1, $o2['id']);
    }

    // ── getOrderById ──────────────────────────────────────────────────────

    public function testGetOrderByIdReturnsCorrectOrder(): void
    {
        $order = $this->repo->placeOrder(['petId' => 1, 'quantity' => 1, 'status' => 'placed']);
        $found = $this->repo->getOrderById($order['id']);
        self::assertSame($order['id'], $found['id']);
    }

    public function testGetOrderByIdReturnsNullForUnknown(): void
    {
        self::assertNull($this->repo->getOrderById(99999));
    }

    // ── deleteOrder ───────────────────────────────────────────────────────

    public function testDeleteOrderRemovesOrder(): void
    {
        $order = $this->repo->placeOrder(['petId' => 1, 'quantity' => 1, 'status' => 'placed']);
        self::assertTrue($this->repo->deleteOrder($order['id']));
        self::assertNull($this->repo->getOrderById($order['id']));
    }

    public function testDeleteOrderReturnsFalseForUnknown(): void
    {
        self::assertFalse($this->repo->deleteOrder(99999));
    }

    // ── getInventory ──────────────────────────────────────────────────────

    public function testGetInventoryReturnsEmptyWhenNoPets(): void
    {
        self::assertSame([], $this->repo->getInventory());
    }

    public function testGetInventoryAggregatesByStatus(): void
    {
        $this->repo->addPet(['name' => 'A', 'photoUrls' => [], 'status' => 'available']);
        $this->repo->addPet(['name' => 'B', 'photoUrls' => [], 'status' => 'pending']);
        $this->repo->addPet(['name' => 'C', 'photoUrls' => [], 'status' => 'available']);
        $inv = $this->repo->getInventory();
        self::assertSame(2, $inv['available']);
        self::assertSame(1, $inv['pending']);
        self::assertArrayNotHasKey('sold', $inv);
    }
}
