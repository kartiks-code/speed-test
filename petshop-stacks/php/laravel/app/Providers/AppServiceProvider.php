<?php

namespace App\Providers;

use App\Repositories\PetstoreRepositoryInterface;
use App\Repositories\PostgresPetstoreRepository;
use Illuminate\Support\ServiceProvider;

class AppServiceProvider extends ServiceProvider
{
    public function register(): void
    {
        // singleton: one repository (and thus one PDO connection) per worker
        // process, instead of reconnecting on every resolution.
        $this->app->singleton(
            PetstoreRepositoryInterface::class,
            PostgresPetstoreRepository::class
        );
    }

    public function boot(): void
    {
        //
    }
}
