namespace Squalr.Engine.Architecture.Assemblers
{
    using System;

    /// <summary>
    /// A class containing the results of an assembler operation.
    /// </summary>
    public class AssemblerResult
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="AssemblerResult" /> class.
        /// </summary>
        /// <param name="bytes">The compiled assembly.</param>
        /// <param name="message">The message of the compilation result.</param>
        /// <param name="innerMessage">The inner message of the compilation result.</param>
        public AssemblerResult(Byte[] bytes, String message, String innerMessage)
        {
            this.Bytes = bytes;
            this.Message = message;
            this.InnerMessage = innerMessage;
        }

        /// <summary>
        /// Gets the bytes from the compiled assembly.
        /// </summary>
        public Byte[] Bytes { get; private set; }

        /// <summary>
        /// Gets the message of the compilation result.
        /// </summary>
        public String Message { get; private set; }

        /// <summary>
        /// Gets the inner message of the compilation result. Usually contains error data.
        /// </summary>
        public String InnerMessage { get; private set; }
    }
    //// End class
}
