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
// Generated by IDLImporter from file nsIPermissionManager.idl
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
    /// This file contains an interface to the Permission Manager,
    /// used to persistenly store permissions for different object types (cookies,
    /// images etc) on a site-by-site basis.
    ///
    /// This service broadcasts the following notification when the permission list
    /// is changed:
    ///
    /// topic  : "perm-changed" (PERM_CHANGE_NOTIFICATION)
    /// broadcast whenever the permission list changes in some way. there
    /// are four possible data strings for this notification; one
    /// notification will be broadcast for each change, and will involve
    /// a single permission.
    /// subject: an nsIPermission interface pointer representing the permission object
    /// that changed.
    /// data   : "deleted"
    /// a permission was deleted. the subject is the deleted permission.
    /// "added"
    /// a permission was added. the subject is the added permission.
    /// "changed"
    /// a permission was changed. the subject is the new permission.
    /// "cleared"
    /// the entire permission list was cleared. the subject is null.
    /// </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("c9fec678-f194-43c9-96b0-7bd9dbdd6bb0")]
	public interface nsIPermissionManager
	{
		
		/// <summary>
        /// Add permission information for a given URI and permission type. This
        /// operation will cause the type string to be registered if it does not
        /// currently exist. If a permission already exists for a given type, it
        /// will be modified.
        ///
        /// @param uri         the uri to add the permission for
        /// @param type        a case-sensitive ASCII string, identifying the consumer.
        /// Consumers should choose this string to be unique, with
        /// respect to other consumers.
        /// @param permission  an integer representing the desired action (e.g. allow
        /// or deny). The interpretation of this number is up to the
        /// consumer, and may represent different actions for different
        /// types. Consumers may use one of the enumerated permission
        /// actions defined above, for convenience.
        /// NOTE: UNKNOWN_ACTION (0) is reserved to represent the
        /// default permission when no entry is found for a host, and
        /// should not be used by consumers to indicate otherwise.
        /// @param expiretype  a constant defining whether this permission should
        /// never expire (EXPIRE_NEVER), expire at the end of the
        /// session (EXPIRE_SESSION), or expire at a specified time
        /// (EXPIRE_TIME).
        /// @param expiretime  an integer representation of when this permission
        /// should be forgotten (milliseconds since Jan 1 1970 0:00:00).
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void Add([MarshalAs(UnmanagedType.Interface)] nsIURI uri, [MarshalAs(UnmanagedType.LPStr)] string type, uint permission, uint expireType, long expireTime);
		
		/// <summary>
        /// Add permission information for a given principal.
        /// It is internally calling the other add() method using the nsIURI from the
        /// principal.
        /// Passing a system principal will be a no-op because they will always be
        /// granted permissions.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void AddFromPrincipal([MarshalAs(UnmanagedType.Interface)] nsIPrincipal principal, [MarshalAs(UnmanagedType.LPStr)] string typed, uint permission, uint expireType, long expireTime);
		
		/// <summary>
        /// Remove permission information for a given host string and permission type.
        /// The host string represents the exact entry in the permission list (such as
        /// obtained from the enumerator), not a URI which that permission might apply
        /// to.
        ///
        /// @param host   the host to remove the permission for
        /// @param type   a case-sensitive ASCII string, identifying the consumer.
        /// The type must have been previously registered using the
        /// add() method.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void Remove([MarshalAs(UnmanagedType.LPStruct)] nsAUTF8StringBase host, [MarshalAs(UnmanagedType.LPStr)] string type);
		
		/// <summary>
        /// Remove permission information for a given principal.
        /// This is internally calling remove() with the host from the principal's URI.
        /// Passing system principal will be a no-op because we never add them to the
        /// database.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void RemoveFromPrincipal([MarshalAs(UnmanagedType.Interface)] nsIPrincipal principal, [MarshalAs(UnmanagedType.LPStr)] string type);
		
		/// <summary>
        /// Clear permission information for all websites.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void RemoveAll();
		
		/// <summary>
        /// Test whether a website has permission to perform the given action.
        /// @param uri     the uri to be tested
        /// @param type    a case-sensitive ASCII string, identifying the consumer
        /// @param return  see add(), param permission. returns UNKNOWN_ACTION when
        /// there is no stored permission for this uri and / or type.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		uint TestPermission([MarshalAs(UnmanagedType.Interface)] nsIURI uri, [MarshalAs(UnmanagedType.LPStr)] string type);
		
		/// <summary>
        /// Test whether the principal has the permission to perform a given action.
        /// System principals will always have permissions granted.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		uint TestPermissionFromPrincipal([MarshalAs(UnmanagedType.Interface)] nsIPrincipal principal, [MarshalAs(UnmanagedType.LPStr)] string type);
		
		/// <summary>
        /// Test whether the principal associated with the window's document has the
        /// permission to perform a given action.  System principals will always
        /// have permissions granted.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		uint TestPermissionFromWindow([MarshalAs(UnmanagedType.Interface)] nsIDOMWindow window, [MarshalAs(UnmanagedType.LPStr)] string type);
		
		/// <summary>
        /// Test whether a website has permission to perform the given action.
        /// This requires an exact hostname match, subdomains are not a match.
        /// @param uri     the uri to be tested
        /// @param type    a case-sensitive ASCII string, identifying the consumer
        /// @param return  see add(), param permission. returns UNKNOWN_ACTION when
        /// there is no stored permission for this uri and / or type.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		uint TestExactPermission([MarshalAs(UnmanagedType.Interface)] nsIURI uri, [MarshalAs(UnmanagedType.LPStr)] string type);
		
		/// <summary>
        /// See testExactPermission() above.
        /// System principals will always have permissions granted.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		uint TestExactPermissionFromPrincipal([MarshalAs(UnmanagedType.Interface)] nsIPrincipal principal, [MarshalAs(UnmanagedType.LPStr)] string type);
		
		/// <summary>
        /// Test whether a website has permission to perform the given action
        /// ignoring active sessions.
        /// System principals will always have permissions granted.
        ///
        /// @param principal the principal
        /// @param type      a case-sensitive ASCII string, identifying the consumer
        /// @param return    see add(), param permission. returns UNKNOWN_ACTION when
        /// there is no stored permission for this uri and / or type.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		uint TestExactPermanentPermission([MarshalAs(UnmanagedType.Interface)] nsIPrincipal principal, [MarshalAs(UnmanagedType.LPStr)] string type);
		
		/// <summary>
        /// Get the permission object associated with the given principal and action.
        /// @param principal The principal
        /// @param type      A case-sensitive ASCII string identifying the consumer
        /// @param exactHost If true, only the specific host will be matched,
        /// @see testExactPermission. If false, subdomains will
        /// also be searched, @see testPermission.
        /// @returns The matching permission object, or null if no matching object
        /// was found. No matching object is equivalent to UNKNOWN_ACTION.
        /// @note Clients in general should prefer the test* methods unless they
        /// need to know the specific stored details.
        /// @note This method will always return null for the system principal.
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIPermission GetPermissionObject([MarshalAs(UnmanagedType.Interface)] nsIPrincipal principal, [MarshalAs(UnmanagedType.LPStr)] string type, [MarshalAs(UnmanagedType.U1)] bool exactHost);
		
		/// <summary>
        /// Increment or decrement our "refcount" of an app id.
        ///
        /// We use this refcount to determine an app's lifetime.  When an app's
        /// refcount goes to 0, we clear the permissions given to the app which are
        /// set to expire at the end of its session.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void AddrefAppId(uint appId);
		
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void ReleaseAppId(uint appId);
		
		/// <summary>
        /// Allows enumeration of all stored permissions
        /// @return an nsISimpleEnumerator interface that allows access to
        /// nsIPermission objects
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsISimpleEnumerator GetEnumeratorAttribute();
		
		/// <summary>
        /// Remove all permissions associated with a given app id.
        /// @param aAppId       The appId of the app
        /// @param aBrowserOnly Whether we should remove permissions associated with
        /// a browser element (true) or all permissions (false).
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void RemovePermissionsForApp(uint appId, [MarshalAs(UnmanagedType.U1)] bool browserOnly);
		
		/// <summary>
        /// If the current permission is set to expire, reset the expiration time. If
        /// there is no permission or the current permission does not expire, this
        /// method will silently return.
        ///
        /// @param sessionExpiretime  an integer representation of when this permission
        /// should be forgotten (milliseconds since
        /// Jan 1 1970 0:00:00), if it is currently
        /// EXPIRE_SESSION.
        /// @param sessionExpiretime  an integer representation of when this permission
        /// should be forgotten (milliseconds since
        /// Jan 1 1970 0:00:00), if it is currently
        /// EXPIRE_TIME.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void UpdateExpireTime([MarshalAs(UnmanagedType.Interface)] nsIPrincipal principal, [MarshalAs(UnmanagedType.LPStr)] string type, [MarshalAs(UnmanagedType.U1)] bool exactHost, ulong sessionExpireTime, ulong persistentExpireTime);
	}
	
	/// <summary>nsIPermissionManagerConsts </summary>
	public class nsIPermissionManagerConsts
	{
		
		// <summary>
        // Predefined return values for the testPermission method and for
        // the permission param of the add method
        // NOTE: UNKNOWN_ACTION (0) is reserved to represent the
        // default permission when no entry is found for a host, and
        // should not be used by consumers to indicate otherwise.
        // </summary>
		public const long UNKNOWN_ACTION = 0;
		
		// 
		public const long ALLOW_ACTION = 1;
		
		// 
		public const long DENY_ACTION = 2;
		
		// 
		public const long PROMPT_ACTION = 3;
		
		// <summary>
        // Predefined expiration types for permissions.  Permissions can be permanent
        // (never expire), expire at the end of the session, or expire at a specified
        // time. Permissions that expire at the end of a session may also have a
        // specified expiration time.
        // </summary>
		public const long EXPIRE_NEVER = 0;
		
		// 
		public const long EXPIRE_SESSION = 1;
		
		// 
		public const long EXPIRE_TIME = 2;
	}
}
