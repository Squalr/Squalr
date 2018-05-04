﻿namespace Squalr.Engine
{
    using System;
    using System.ComponentModel;
    using System.Threading.Tasks;

    public delegate void OnProgressUpdate(Single progress);
    public delegate void OnTaskCompleted(TrackableTask task);

    public class TrackableTask : INotifyPropertyChanged
    {
        private Single progress;

        private String name;

        private Boolean isCanceled;

        private Boolean isCompleted;

        protected TrackableTask()
        {
        }

        public OnTaskCompleted TaskCompletedCallback { get; set; }

        public Single Progress
        {
            get
            {
                return this.progress;
            }

            set
            {
                this.progress = value;
                this.RaisePropertyChanged(nameof(this.Progress));
            }
        }

        public String Name
        {
            get
            {
                return this.name;
            }

            set
            {
                this.name = value;
                this.RaisePropertyChanged(nameof(this.Name));
            }
        }

        public Boolean IsCanceled
        {
            get
            {
                return this.isCanceled;
            }

            protected set
            {
                this.isCanceled = value;
                this.RaisePropertyChanged(nameof(this.IsCanceled));
            }
        }

        public Boolean IsCompleted
        {
            get
            {
                return this.isCompleted;
            }

            protected set
            {
                this.isCompleted = value;
                this.RaisePropertyChanged(nameof(this.IsCompleted));
                this.TaskCompletedCallback?.Invoke(this);
            }
        }

        public event PropertyChangedEventHandler PropertyChanged;

        /// <summary>
        /// Indicates that a given property in this project item has changed.
        /// </summary>
        /// <param name="propertyName">The name of the changed property.</param>
        protected void RaisePropertyChanged(String propertyName)
        {
            this.PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }

    public class TrackableTask<T> : TrackableTask
    {
        public TrackableTask(String name)
        {
            this.Name = name;
        }

        public T Result
        {
            get
            {
                return this.Task.Result;
            }
        }

        private Task<T> Task { get; set; }

        public void SetTrackedTask(Task<T> task)
        {
            this.Task = task;

            this.AwaitCompletion();
        }

        public void UpdateProgress(Single progress)
        {
            this.Progress = progress;
        }

        private async void AwaitCompletion()
        {
            await this.Task;

            this.IsCompleted = true;
        }
    }
    //// End class
}
//// End namespace