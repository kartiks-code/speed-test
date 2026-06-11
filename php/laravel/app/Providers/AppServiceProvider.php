<?php

namespace App\Providers;

use App\Repositories\PetstoreRepositoryInterface;
use App\Repositories\PostgresPetstoreRepository;
use Illuminate\Support\ServiceProvider;

class AppServiceProvider extends ServiceProvider
{
    public function register(): void
    {
        $this->app->bind(
            PetstoreRepositoryInterface::class,
            PostgresPetstoreRepository::class
        );
    }

    public function boot(): void
    {
        //
    }
}
