// --------------------------------------------------------------------------------------------
// Version: MPL 1.1/GPL 2.0/LGPL 2.1
// 
// The contents of this file are subject to the Mozilla Public License Version
// 1.1 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
// http://www.mozilla.org/MPL/
// 
// Software distributed under the License is distributed on an "AS IS" basis,
// WITHOUT WARRANTY OF ANY KIND, either express or implied. See the License
// for the specific language governing rights and limitations under the
// License.
// 
// <remarks>
// Generated by IDLImporter from file nsIHashable.idl
// 
// You should use these interfaces when you access the COM objects defined in the mentioned
// IDL/IDH file.
// </remarks>
// --------------------------------------------------------------------------------------------
namespace Gecko
{
    using System;
    using System.Runtime.CompilerServices;
    using System.Runtime.InteropServices;


    /// <summary>
    /// Represents an object that can be stored in a hashtable.
    /// </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("17e595fa-b57a-4933-bd0f-b1812e8ab188")]
	public interface nsIHashable
	{
		
		/// <summary>
        /// Is this object the equivalent of the other object?
        /// </summary>
		[return: MarshalAs(UnmanagedType.U1)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		bool Equals([MarshalAs(UnmanagedType.Interface)] nsIHashable aOther);
		
		/// <summary>
        /// A generated hashcode for this object. Objects that are equivalent
        /// must have the same hash code. Getting this property should never
        /// throw an exception!
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		uint GetHashCodeAttribute();
	}
}
