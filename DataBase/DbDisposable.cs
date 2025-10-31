using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Storage;

namespace DataBase;

public class DbDisposable<T> : IAsyncDisposable where T : DbContext, new()
{
    protected readonly T _context;
    private IDbContextTransaction? _transaction;
    private bool _disposed;

    public T Context => _context;

    protected DbDisposable() : this(new T()) { }

    protected DbDisposable(T dbContext)
    {
        _context = dbContext;
    }

    public async Task SaveChangesAsync()
    {
        await _context.SaveChangesAsync();
    }

    public async Task BeginTransactionAsync()
    {
        if (_transaction != null) return;
        _transaction = await _context.Database.BeginTransactionAsync();
    }

    public async Task CommitAsync()
    {
        if (_transaction == null) return;
        await _transaction.CommitAsync();
        await _transaction.DisposeAsync();
        _transaction = null;
    }

    public async Task RollbackAsync()
    {
        if (_transaction == null) return;
        await _transaction.RollbackAsync();
        await _transaction.DisposeAsync();
        _transaction = null;
    }

    public T GetDbContext()
    {
        return _context;
    }

    public async ValueTask DisposeAsync()
    {
        if (_disposed) return;

        if (_transaction != null)
        {
            await _transaction.DisposeAsync();
            _transaction = null;
        }

        await _context.DisposeAsync();
        _disposed = true;
    }
}
