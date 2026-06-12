<?php

declare(strict_types=1);

use App\Http\Controllers\PetController;
use App\Http\Controllers\StoreController;
use App\Http\Controllers\UserController;
use Illuminate\Support\Facades\Route;

// ── Pet ───────────────────────────────────────────────────────────────────────

Route::post('/pet', [PetController::class, 'addPet']);
Route::put('/pet', [PetController::class, 'updatePet']);
Route::get('/pet/findByStatus', [PetController::class, 'findPetsByStatus']);
Route::get('/pet/findByTags', [PetController::class, 'findPetsByTags']);
Route::get('/pet/{petId}', [PetController::class, 'getPetById']);
Route::post('/pet/{petId}', [PetController::class, 'updatePetWithForm']);
Route::delete('/pet/{petId}', [PetController::class, 'deletePet']);
Route::post('/pet/{petId}/uploadImage', [PetController::class, 'uploadFile']);

// ── Store ─────────────────────────────────────────────────────────────────────

Route::get('/store/inventory', [StoreController::class, 'getInventory']);
Route::post('/store/order', [StoreController::class, 'placeOrder']);
Route::get('/store/order/{orderId}', [StoreController::class, 'getOrderById']);
Route::delete('/store/order/{orderId}', [StoreController::class, 'deleteOrder']);

// ── User ──────────────────────────────────────────────────────────────────────

Route::post('/user', [UserController::class, 'createUser']);
Route::post('/user/createWithList', [UserController::class, 'createUsersWithListInput']);
Route::get('/user/login', [UserController::class, 'loginUser']);
Route::get('/user/logout', [UserController::class, 'logoutUser']);
Route::get('/user/{username}', [UserController::class, 'getUserByName']);
Route::put('/user/{username}', [UserController::class, 'updateUser']);
Route::delete('/user/{username}', [UserController::class, 'deleteUser']);
