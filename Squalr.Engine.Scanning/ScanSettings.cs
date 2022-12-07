namespace Squalr.Engine.Scanning
{
    using Squalr.Engine.Common;
    using System;
    
    public static class ScanSettings
    {
        public static Int32 ResultReadInterval
        {
            get
            {
                return Properties.Settings.Default.ResultReadInterval;
            }

            set
            {
                Properties.Settings.Default.ResultReadInterval = value;
            }
        }

        public static Int32 TableReadInterval
        {
            get
            {
                return Properties.Settings.Default.TableReadInterval;
            }

            set
            {
                Properties.Settings.Default.TableReadInterval = value;
            }
        }

        public static Int32 FreezeInterval
        {
            get
            {
                return Properties.Settings.Default.FreezeInterval;
            }

            set
            {
                Properties.Settings.Default.FreezeInterval = value;
            }
        }

        public static Boolean MemoryTypeNone
        {
            get
            {
                return Properties.Settings.Default.MemoryTypeNone;
            }

            set
            {
                Properties.Settings.Default.MemoryTypeNone = value;
            }
        }

        public static Boolean MemoryTypePrivate
        {
            get
            {
                return Properties.Settings.Default.MemoryTypePrivate;
            }

            set
            {
                Properties.Settings.Default.MemoryTypePrivate = value;
            }
        }

        public static Boolean MemoryTypeImage
        {
            get
            {
                return Properties.Settings.Default.MemoryTypeImage;
            }

            set
            {
                Properties.Settings.Default.MemoryTypeImage = value;
            }
        }

        public static Boolean MemoryTypeMapped
        {
            get
            {
                return Properties.Settings.Default.MemoryTypeMapped;
            }

            set
            {
                Properties.Settings.Default.MemoryTypeMapped = value;
            }
        }

        public static MemoryAlignment Alignment
        {
            get
            {
                return (MemoryAlignment)Properties.Settings.Default.Alignment;
            }

            set
            {
                Properties.Settings.Default.Alignment = (int)value;
            }
        }

        public static Boolean RequiredWrite
        {
            get
            {
                return Properties.Settings.Default.RequiredWrite;
            }

            set
            {
                Properties.Settings.Default.RequiredWrite = value;
            }
        }

        public static Boolean RequiredExecute
        {
            get
            {
                return Properties.Settings.Default.RequiredExecute;
            }

            set
            {
                Properties.Settings.Default.RequiredExecute = value;
            }
        }

        public static Boolean RequiredCopyOnWrite
        {
            get
            {
                return Properties.Settings.Default.RequiredCopyOnWrite;
            }

            set
            {
                Properties.Settings.Default.RequiredCopyOnWrite = value;
            }
        }

        public static Boolean ExcludedWrite
        {
            get
            {
                return Properties.Settings.Default.ExcludedWrite;
            }

            set
            {
                Properties.Settings.Default.ExcludedWrite = value;
            }
        }

        public static Boolean ExcludedExecute
        {
            get
            {
                return Properties.Settings.Default.ExcludedExecute;
            }

            set
            {
                Properties.Settings.Default.ExcludedExecute = value;
            }
        }

        public static Boolean ExcludedCopyOnWrite
        {
            get
            {
                return Properties.Settings.Default.ExcludedCopyOnWrite;
            }

            set
            {
                Properties.Settings.Default.ExcludedCopyOnWrite = value;
            }
        }

        public static UInt64 StartAddress
        {
            get
            {
                return Properties.Settings.Default.StartAddress;
            }

            set
            {
                Properties.Settings.Default.StartAddress = value;
            }
        }

        public static UInt64 EndAddress
        {
            get
            {
                return Properties.Settings.Default.EndAddress;
            }

            set
            {
                Properties.Settings.Default.EndAddress = value;
            }
        }

        public static Boolean IsUserMode
        {
            get
            {
                return Properties.Settings.Default.IsUserMode;
            }

            set
            {
                Properties.Settings.Default.IsUserMode = value;
            }
        }

        public static ScannableType DataType
        {
            get
            {
                return Properties.Settings.Default.DataType;
            }

            set
            {
                Properties.Settings.Default.DataType = value;
            }
        }

        public static EmulatorType EmulatorType
        {
            get
            {
                return (EmulatorType)Properties.Settings.Default.EmulatorType;
            }

            set
            {
                Properties.Settings.Default.EmulatorType = (int)value;
            }
        }

        public static Boolean UseMultiThreadScans
        {
            get
            {
                return Properties.Settings.Default.UseMultiThreadScans;
            }

            set
            {
                Properties.Settings.Default.UseMultiThreadScans = value;
            }
        }

        public static MemoryAlignment ResolveAutoAlignment(MemoryAlignment alignment, Int32 dataTypeSize)
        {
            return alignment == MemoryAlignment.Auto ? (MemoryAlignment)dataTypeSize : alignment;
        }
    }
    //// End class
}
//// End namespace