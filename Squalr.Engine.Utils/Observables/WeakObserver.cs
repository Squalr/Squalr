namespace Squalr.Engine.Common.Observables
{
    using System;

    public class WeakObserver<T> : IDisposable, IObserver<T>
    {
        private readonly WeakReference reference;

        private readonly IDisposable subscription;

        private Boolean disposed;

        public WeakObserver(IObservable<T> observable, IObserver<T> observer)
        {
            this.reference = new WeakReference(observer);
            this.subscription = observable.Subscribe(this);
        }

        /// <summary>
        /// Notifies the observer that the provider has finished sending push-based notifications.
        /// </summary>
        void IObserver<T>.OnCompleted()
        {
            IObserver<T> observer = (IObserver<T>)this.reference.Target;

            if (observer != null)
            {
                observer.OnCompleted();
            }
            else
            {
                this.Dispose();
            }
        }

        /// <summary>
        /// Notifies the observer that the provider has experienced an error condition.
        /// </summary>
        /// <param name="error">An object that provides additional information about the error.</param>
        void IObserver<T>.OnError(Exception error)
        {
            IObserver<T> observer = (IObserver<T>)this.reference.Target;

            if (observer != null)
            {
                observer.OnError(error);
            }
            else
            {
                this.Dispose();
            }
        }

        /// <summary>
        /// Subscription event for when an event is fired.
        /// </summary>
        /// <param name="value">The value associated with this event.</param>
        void IObserver<T>.OnNext(T value)
        {
            IObserver<T> observer = (IObserver<T>)this.reference.Target;

            if (observer != null)
            {
                observer.OnNext(value);
            }
            else
            {
                this.Dispose();
            }
        }

        public void Dispose()
        {
            if (!this.disposed)
            {
                this.disposed = true;
                this.subscription.Dispose();
            }
        }
    }
    //// End class
}
//// End namespace