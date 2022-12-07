namespace Squalr.Engine.Common.Hardware
{
    using System;
    using System.Numerics;

    /// <summary>
    /// A class containing convenience methods and properties for hardware vectors.
    /// </summary>
    public static class Vectors
    {
        /// <summary>
        /// A vector with all bits set to 1. TODO: If C# ever adds support for extension properties, this would be great to offload onto all Vector{T} types.
        /// </summary>
        public static readonly Vector<Byte> AllBits = Vector.OnesComplement(Vector<Byte>.Zero);

        /// <summary>
        /// Initializes static members of the <see cref="Vectors" /> class.
        /// </summary>
        static Vectors()
        {
            Vectors.HasVectorSupport = Vector.IsHardwareAccelerated;
            Vectors.VectorSize = Vector<Byte>.Count;
        }

        /// <summary>
        /// Gets a value indicating whether the archiecture has vector instruction support.
        /// </summary>
        public static Boolean HasVectorSupport { get; private set; }

        /// <summary>
        /// Gets the vector size supported by the current architecture.
        /// If vectors are not supported, returns the lowest common denominator vector size for the architecture.
        /// </summary>
        public static Int32 VectorSize { get; private set; }
    }
    //// End class
}
//// End namespace