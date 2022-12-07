namespace Squalr.Engine.Common.Extensions
{
    using System;

    public static class RandomExtensions
    {
        public static Int16 RandomInt16(this Random self, Int16 min = 0, Int16 max = Int16.MaxValue)
        {
            Byte[] buffer = new Byte[2];
            self.NextBytes(buffer);
            return min == max ? min : unchecked((Int16)(Math.Abs(BitConverter.ToInt16(buffer, 0) % (max - min)) + min));
        }

        public static UInt16 RandomUInt16(this Random self, UInt16 min = 0, UInt16 max = UInt16.MaxValue)
        {
            Byte[] buffer = new Byte[2];
            self.NextBytes(buffer);
            return min == max ? min : unchecked((UInt16)((BitConverter.ToUInt16(buffer, 0) % (max - min)) + min));
        }

        public static Int32 RandomInt32(this Random self, Int16 min = 0, Int16 max = Int16.MaxValue)
        {
            Byte[] buffer = new Byte[4];
            self.NextBytes(buffer);
            return min == max ? min : (Math.Abs(BitConverter.ToInt32(buffer, 0) % (max - min)) + min);
        }

        public static UInt32 RandomUInt32(this Random self, UInt32 min = 0, UInt32 max = UInt32.MaxValue)
        {
            Byte[] buffer = new Byte[4];
            self.NextBytes(buffer);
            return min == max ? min : ((BitConverter.ToUInt32(buffer, 0) % (max - min)) + min);
        }

        public static Int64 RandomInt64(this Random self, Int64 min = 0, Int64 max = Int64.MaxValue)
        {
            Byte[] buffer = new Byte[8];
            self.NextBytes(buffer);
            return min == max ? min : (Math.Abs(BitConverter.ToInt64(buffer, 0) % (max - min)) + min);
        }

        public static UInt64 RandomUInt64(this Random self, UInt64 min = 0, UInt64 max = UInt64.MaxValue)
        {
            Byte[] buffer = new Byte[8];
            self.NextBytes(buffer);
            return min == max ? min : ((BitConverter.ToUInt64(buffer, 0) % (max - min)) + min);
        }
    }
    //// End calss
}
//// End namespace